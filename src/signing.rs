use anyhow::Result;
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::SigningConfig;
use crate::storage::StorageBackend;

/// Content signing and verification service for drift registry
/// Supports multiple signature formats: Cosign, Notary v2, and custom signatures
#[derive(Clone)]
pub struct SigningService {
    config: SigningConfig,
    storage: Arc<dyn StorageBackend>,
    key_store: Arc<RwLock<KeyStore>>,
    signature_cache: Arc<RwLock<HashMap<String, CachedSignature>>>,
}

/// Key store for managing signing keys and certificates
#[derive(Debug)]
pub struct KeyStore {
    pub signing_keys: HashMap<String, SigningKey>,
    pub verification_keys: HashMap<String, VerificationKey>,
    pub trust_stores: HashMap<String, TrustStore>,
}

/// Signing key with private key material
#[derive(Debug, Clone)]
pub struct SigningKey {
    pub key_id: String,
    pub algorithm: SignatureAlgorithm,
    pub private_key: Vec<u8>, // DER encoded private key
    pub certificate: Option<Vec<u8>>, // DER encoded certificate
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Verification key with public key material
#[derive(Debug, Clone)]
pub struct VerificationKey {
    pub key_id: String,
    pub algorithm: SignatureAlgorithm,
    pub public_key: Vec<u8>, // DER encoded public key
    pub certificate: Option<Vec<u8>>, // DER encoded certificate
    pub trusted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Trust store containing trusted root certificates
#[derive(Debug, Clone)]
pub struct TrustStore {
    pub name: String,
    pub root_certificates: Vec<Vec<u8>>, // DER encoded certificates
    pub intermediate_certificates: Vec<Vec<u8>>,
    pub revoked_certificates: Vec<String>, // Serial numbers
}

/// Supported signature algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureAlgorithm {
    #[serde(rename = "rsa-pss-sha256")]
    RsaPssSha256,
    #[serde(rename = "rsa-pkcs1-sha256")]
    RsaPkcs1Sha256,
    #[serde(rename = "ecdsa-p256-sha256")]
    EcdsaP256Sha256,
    #[serde(rename = "ecdsa-p384-sha384")]
    EcdsaP384Sha384,
    #[serde(rename = "ed25519")]
    Ed25519,
}

/// Signature formats supported by the registry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureFormat {
    /// Cosign signatures (ECDSA/RSA + JSON)
    #[serde(rename = "cosign")]
    Cosign,
    /// Notary v2 signatures (JWS)
    #[serde(rename = "notary-v2")]
    NotaryV2,
    /// Simple JSON signatures
    #[serde(rename = "simple")]
    Simple,
    /// In-Toto attestations
    #[serde(rename = "in-toto")]
    InToto,
}

/// Content signature containing all signature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSignature {
    pub signature_id: String,
    pub content_digest: String, // SHA256 digest of signed content
    pub format: SignatureFormat,
    pub algorithm: SignatureAlgorithm,
    pub signature: Vec<u8>, // Raw signature bytes
    pub key_id: String,
    pub certificate_chain: Option<Vec<Vec<u8>>>, // DER encoded certificates
    pub payload: SignaturePayload,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Signature payload containing metadata about signed content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignaturePayload {
    /// Content identifier (manifest digest, blob digest, etc.)
    pub subject: String,
    /// Content type (manifest, blob, attestation)
    pub content_type: String,
    /// Registry repository name
    pub repository: String,
    /// Optional tag reference
    pub tag: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp when content was signed
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cached signature for performance optimization
#[derive(Debug, Clone)]
pub struct CachedSignature {
    pub signature: ContentSignature,
    pub verification_result: VerificationResult,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// Result of signature verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub trusted: bool,
    pub key_id: String,
    pub algorithm: SignatureAlgorithm,
    pub verified_at: chrono::DateTime<chrono::Utc>,
    pub certificate_chain_valid: Option<bool>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Signature verification policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPolicy {
    pub require_signatures: bool,
    pub required_signatures_count: usize,
    pub allowed_signature_formats: Vec<SignatureFormat>,
    pub allowed_algorithms: Vec<SignatureAlgorithm>,
    pub trust_stores: Vec<String>,
    pub require_certificate_chain: bool,
    pub allow_self_signed: bool,
    pub max_signature_age_hours: Option<u64>,
}

/// Trait for signature verification backends
#[async_trait]
pub trait SignatureVerifier: Send + Sync {
    async fn verify_signature(
        &self,
        content: &[u8],
        signature: &ContentSignature,
        policy: &VerificationPolicy,
    ) -> Result<VerificationResult>;
}

impl SigningService {
    pub async fn new(
        config: SigningConfig,
        storage: Arc<dyn StorageBackend>,
    ) -> Result<Self> {
        info!("Initializing content signing service");

        let key_store = Arc::new(RwLock::new(KeyStore {
            signing_keys: HashMap::new(),
            verification_keys: HashMap::new(),
            trust_stores: HashMap::new(),
        }));

        let service = Self {
            config,
            storage,
            key_store,
            signature_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load keys and trust stores from configuration
        service.initialize_key_store().await?;

        info!("Content signing service initialized successfully");
        Ok(service)
    }

    /// Initialize key store with configured keys and trust stores
    async fn initialize_key_store(&self) -> Result<()> {
        let mut key_store = self.key_store.write().await;

        // Load signing keys
        for key_config in &self.config.signing_keys {
            let signing_key = self.load_signing_key(key_config).await?;
            key_store.signing_keys.insert(signing_key.key_id.clone(), signing_key);
        }

        // Load verification keys
        for key_config in &self.config.verification_keys {
            let verification_key = self.load_verification_key(key_config).await?;
            key_store.verification_keys.insert(verification_key.key_id.clone(), verification_key);
        }

        // Load trust stores
        for trust_config in &self.config.trust_stores {
            let trust_store = self.load_trust_store(trust_config).await?;
            key_store.trust_stores.insert(trust_store.name.clone(), trust_store);
        }

        info!("Loaded {} signing keys, {} verification keys, {} trust stores",
            key_store.signing_keys.len(),
            key_store.verification_keys.len(),
            key_store.trust_stores.len()
        );

        Ok(())
    }

    /// Sign content with specified key and format
    pub async fn sign_content(
        &self,
        content: &[u8],
        key_id: &str,
        format: SignatureFormat,
        payload: SignaturePayload,
    ) -> Result<ContentSignature> {
        debug!("Signing content with key {} in format {:?}", key_id, format);

        // Get signing key
        let key_store = self.key_store.read().await;
        let signing_key = key_store.signing_keys.get(key_id)
            .ok_or_else(|| anyhow::anyhow!("Signing key not found: {}", key_id))?;

        // Calculate content digest
        let content_digest = hex::encode(Sha256::digest(content));

        // Create signature based on format
        let signature_bytes = match format {
            SignatureFormat::Cosign => self.create_cosign_signature(content, signing_key, &payload).await?,
            SignatureFormat::NotaryV2 => self.create_notary_v2_signature(content, signing_key, &payload).await?,
            SignatureFormat::Simple => self.create_simple_signature(content, signing_key, &payload).await?,
            SignatureFormat::InToto => self.create_in_toto_signature(content, signing_key, &payload).await?,
        };

        let signature = ContentSignature {
            signature_id: uuid::Uuid::new_v4().to_string(),
            content_digest,
            format,
            algorithm: signing_key.algorithm.clone(),
            signature: signature_bytes,
            key_id: key_id.to_string(),
            certificate_chain: signing_key.certificate.as_ref().map(|cert| vec![cert.clone()]),
            payload,
            created_at: chrono::Utc::now(),
            expires_at: signing_key.expires_at,
        };

        // Store signature
        self.store_signature(&signature).await?;

        info!("Content signed successfully with signature ID: {}", signature.signature_id);
        Ok(signature)
    }

    /// Verify content signature
    pub async fn verify_signature(
        &self,
        content: &[u8],
        signature: &ContentSignature,
        policy: &VerificationPolicy,
    ) -> Result<VerificationResult> {
        debug!("Verifying signature {} for content", signature.signature_id);

        // Check cache first
        if let Some(cached) = self.get_cached_verification(&signature.signature_id).await {
            if chrono::Utc::now().signed_duration_since(cached.cached_at).num_minutes() < 5 {
                debug!("Using cached verification result");
                return Ok(cached.verification_result);
            }
        }

        // Verify content digest matches
        let content_digest = hex::encode(Sha256::digest(content));
        if content_digest != signature.content_digest {
            return Ok(VerificationResult {
                valid: false,
                trusted: false,
                key_id: signature.key_id.clone(),
                algorithm: signature.algorithm.clone(),
                verified_at: chrono::Utc::now(),
                certificate_chain_valid: None,
                errors: vec!["Content digest mismatch".to_string()],
                warnings: vec![],
            });
        }

        // Check signature format is allowed
        if !policy.allowed_signature_formats.contains(&signature.format) {
            return Ok(VerificationResult {
                valid: false,
                trusted: false,
                key_id: signature.key_id.clone(),
                algorithm: signature.algorithm.clone(),
                verified_at: chrono::Utc::now(),
                certificate_chain_valid: None,
                errors: vec![format!("Signature format {:?} not allowed", signature.format)],
                warnings: vec![],
            });
        }

        // Check signature algorithm is allowed
        if !policy.allowed_algorithms.contains(&signature.algorithm) {
            return Ok(VerificationResult {
                valid: false,
                trusted: false,
                key_id: signature.key_id.clone(),
                algorithm: signature.algorithm.clone(),
                verified_at: chrono::Utc::now(),
                certificate_chain_valid: None,
                errors: vec![format!("Signature algorithm {:?} not allowed", signature.algorithm)],
                warnings: vec![],
            });
        }

        // Verify signature based on format
        let result = match signature.format {
            SignatureFormat::Cosign => self.verify_cosign_signature(content, signature, policy).await?,
            SignatureFormat::NotaryV2 => self.verify_notary_v2_signature(content, signature, policy).await?,
            SignatureFormat::Simple => self.verify_simple_signature(content, signature, policy).await?,
            SignatureFormat::InToto => self.verify_in_toto_signature(content, signature, policy).await?,
        };

        // Cache result
        self.cache_verification_result(&signature.signature_id, signature.clone(), result.clone()).await;

        info!("Signature verification completed: valid={}, trusted={}", result.valid, result.trusted);
        Ok(result)
    }

    /// Get all signatures for a piece of content
    pub async fn get_content_signatures(&self, content_digest: &str) -> Result<Vec<ContentSignature>> {
        debug!("Getting signatures for content digest: {}", content_digest);

        let key = format!("signatures/{}", content_digest);
        match self.storage.get_blob(&key).await? {
            Some(data) => {
                let signatures: Vec<ContentSignature> = serde_json::from_slice(&data)?;
                Ok(signatures)
            }
            None => Ok(vec![]),
        }
    }

    /// Store signature in storage backend
    async fn store_signature(&self, signature: &ContentSignature) -> Result<()> {
        // Store individual signature
        let signature_key = format!("signatures/{}/sig_{}", signature.content_digest, signature.signature_id);
        let signature_data = serde_json::to_vec(signature)?;
        self.storage.put_blob(&signature_key, signature_data.into()).await?;

        // Update content signatures list
        let mut signatures = self.get_content_signatures(&signature.content_digest).await?;
        signatures.push(signature.clone());

        let list_key = format!("signatures/{}", signature.content_digest);
        let list_data = serde_json::to_vec(&signatures)?;
        self.storage.put_blob(&list_key, list_data.into()).await?;

        Ok(())
    }

    /// Create Cosign-compatible signature
    async fn create_cosign_signature(
        &self,
        content: &[u8],
        signing_key: &SigningKey,
        payload: &SignaturePayload,
    ) -> Result<Vec<u8>> {
        debug!("Creating Cosign signature");

        // Create Cosign-compatible payload
        let cosign_payload = serde_json::json!({
            "critical": {
                "identity": {
                    "docker-reference": format!("{}/{}", payload.repository, payload.subject)
                },
                "image": {
                    "docker-manifest-digest": payload.subject
                },
                "type": "cosign container image signature"
            },
            "optional": {
                "timestamp": payload.timestamp.timestamp(),
                "tag": payload.tag,
                "metadata": payload.metadata
            }
        });

        let payload_bytes = serde_json::to_vec(&cosign_payload)?;

        // Sign the payload
        self.sign_bytes(&payload_bytes, signing_key).await
    }

    /// Create Notary v2 signature (JWS)
    async fn create_notary_v2_signature(
        &self,
        content: &[u8],
        signing_key: &SigningKey,
        payload: &SignaturePayload,
    ) -> Result<Vec<u8>> {
        debug!("Creating Notary v2 signature");

        // Create JWS header
        let header = serde_json::json!({
            "alg": self.algorithm_to_jws_alg(&signing_key.algorithm),
            "typ": "JWT",
            "kid": signing_key.key_id
        });

        // Create JWS payload
        let jws_payload = serde_json::json!({
            "sub": payload.subject,
            "iat": payload.timestamp.timestamp(),
            "exp": payload.timestamp.timestamp() + 3600, // 1 hour
            "iss": "drift-registry",
            "nbf": payload.timestamp.timestamp(),
            "repository": payload.repository,
            "tag": payload.tag,
            "content_type": payload.content_type,
            "metadata": payload.metadata
        });

        // Encode JWS
        let header_b64 = general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?);
        let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&jws_payload)?);
        let signing_input = format!("{}.{}", header_b64, payload_b64);

        let signature_bytes = self.sign_bytes(signing_input.as_bytes(), signing_key).await?;
        let signature_b64 = general_purpose::URL_SAFE_NO_PAD.encode(signature_bytes);

        let jws = format!("{}.{}.{}", header_b64, payload_b64, signature_b64);
        Ok(jws.into_bytes())
    }

    /// Create simple JSON signature
    async fn create_simple_signature(
        &self,
        content: &[u8],
        signing_key: &SigningKey,
        payload: &SignaturePayload,
    ) -> Result<Vec<u8>> {
        debug!("Creating simple signature");

        let content_digest = hex::encode(Sha256::digest(content));
        let signing_input = format!("{}:{}", content_digest, serde_json::to_string(payload)?);

        self.sign_bytes(signing_input.as_bytes(), signing_key).await
    }

    /// Create in-toto attestation signature
    async fn create_in_toto_signature(
        &self,
        content: &[u8],
        signing_key: &SigningKey,
        payload: &SignaturePayload,
    ) -> Result<Vec<u8>> {
        debug!("Creating in-toto signature");

        let attestation = serde_json::json!({
            "_type": "link",
            "subject": [{
                "name": payload.subject,
                "digest": {
                    "sha256": hex::encode(Sha256::digest(content))
                }
            }],
            "predicateType": "https://in-toto.io/Statement/v0.1",
            "predicate": {
                "repository": payload.repository,
                "tag": payload.tag,
                "content_type": payload.content_type,
                "timestamp": payload.timestamp,
                "metadata": payload.metadata
            }
        });

        let attestation_bytes = serde_json::to_vec(&attestation)?;
        self.sign_bytes(&attestation_bytes, signing_key).await
    }

    /// Sign bytes with the given key
    async fn sign_bytes(&self, data: &[u8], signing_key: &SigningKey) -> Result<Vec<u8>> {
        match signing_key.algorithm {
            SignatureAlgorithm::RsaPssSha256 => {
                // TODO: Implement RSA-PSS signature
                warn!("RSA-PSS signature not implemented, using placeholder");
                Ok(format!("RSA-PSS-SHA256:placeholder-signature-for:{}", hex::encode(Sha256::digest(data))).into_bytes())
            }
            SignatureAlgorithm::RsaPkcs1Sha256 => {
                // TODO: Implement RSA PKCS#1 signature
                warn!("RSA PKCS#1 signature not implemented, using placeholder");
                Ok(format!("RSA-PKCS1-SHA256:placeholder-signature-for:{}", hex::encode(Sha256::digest(data))).into_bytes())
            }
            SignatureAlgorithm::EcdsaP256Sha256 => {
                // TODO: Implement ECDSA P-256 signature
                warn!("ECDSA P-256 signature not implemented, using placeholder");
                Ok(format!("ECDSA-P256-SHA256:placeholder-signature-for:{}", hex::encode(Sha256::digest(data))).into_bytes())
            }
            SignatureAlgorithm::EcdsaP384Sha384 => {
                // TODO: Implement ECDSA P-384 signature
                warn!("ECDSA P-384 signature not implemented, using placeholder");
                Ok(format!("ECDSA-P384-SHA384:placeholder-signature-for:{}", hex::encode(Sha256::digest(data))).into_bytes())
            }
            SignatureAlgorithm::Ed25519 => {
                // TODO: Implement Ed25519 signature
                warn!("Ed25519 signature not implemented, using placeholder");
                Ok(format!("ED25519:placeholder-signature-for:{}", hex::encode(Sha256::digest(data))).into_bytes())
            }
        }
    }

    /// Convert signature algorithm to JWS algorithm identifier
    fn algorithm_to_jws_alg(&self, algorithm: &SignatureAlgorithm) -> &'static str {
        match algorithm {
            SignatureAlgorithm::RsaPssSha256 => "PS256",
            SignatureAlgorithm::RsaPkcs1Sha256 => "RS256",
            SignatureAlgorithm::EcdsaP256Sha256 => "ES256",
            SignatureAlgorithm::EcdsaP384Sha384 => "ES384",
            SignatureAlgorithm::Ed25519 => "EdDSA",
        }
    }

    /// Verify Cosign signature
    async fn verify_cosign_signature(
        &self,
        _content: &[u8],
        signature: &ContentSignature,
        _policy: &VerificationPolicy,
    ) -> Result<VerificationResult> {
        // TODO: Implement actual Cosign signature verification
        warn!("Cosign signature verification not fully implemented");

        Ok(VerificationResult {
            valid: true, // Placeholder
            trusted: false,
            key_id: signature.key_id.clone(),
            algorithm: signature.algorithm.clone(),
            verified_at: chrono::Utc::now(),
            certificate_chain_valid: None,
            errors: vec![],
            warnings: vec!["Cosign verification not fully implemented".to_string()],
        })
    }

    /// Verify Notary v2 signature
    async fn verify_notary_v2_signature(
        &self,
        _content: &[u8],
        signature: &ContentSignature,
        _policy: &VerificationPolicy,
    ) -> Result<VerificationResult> {
        // TODO: Implement actual Notary v2 signature verification
        warn!("Notary v2 signature verification not fully implemented");

        Ok(VerificationResult {
            valid: true, // Placeholder
            trusted: false,
            key_id: signature.key_id.clone(),
            algorithm: signature.algorithm.clone(),
            verified_at: chrono::Utc::now(),
            certificate_chain_valid: None,
            errors: vec![],
            warnings: vec!["Notary v2 verification not fully implemented".to_string()],
        })
    }

    /// Verify simple signature
    async fn verify_simple_signature(
        &self,
        _content: &[u8],
        signature: &ContentSignature,
        _policy: &VerificationPolicy,
    ) -> Result<VerificationResult> {
        // TODO: Implement actual simple signature verification
        warn!("Simple signature verification not fully implemented");

        Ok(VerificationResult {
            valid: true, // Placeholder
            trusted: false,
            key_id: signature.key_id.clone(),
            algorithm: signature.algorithm.clone(),
            verified_at: chrono::Utc::now(),
            certificate_chain_valid: None,
            errors: vec![],
            warnings: vec!["Simple signature verification not fully implemented".to_string()],
        })
    }

    /// Verify in-toto signature
    async fn verify_in_toto_signature(
        &self,
        _content: &[u8],
        signature: &ContentSignature,
        _policy: &VerificationPolicy,
    ) -> Result<VerificationResult> {
        // TODO: Implement actual in-toto signature verification
        warn!("In-toto signature verification not fully implemented");

        Ok(VerificationResult {
            valid: true, // Placeholder
            trusted: false,
            key_id: signature.key_id.clone(),
            algorithm: signature.algorithm.clone(),
            verified_at: chrono::Utc::now(),
            certificate_chain_valid: None,
            errors: vec![],
            warnings: vec!["In-toto verification not fully implemented".to_string()],
        })
    }

    /// Cache verification result
    async fn cache_verification_result(
        &self,
        signature_id: &str,
        signature: ContentSignature,
        result: VerificationResult,
    ) {
        let mut cache = self.signature_cache.write().await;
        cache.insert(signature_id.to_string(), CachedSignature {
            signature,
            verification_result: result,
            cached_at: chrono::Utc::now(),
        });
    }

    /// Get cached verification result
    async fn get_cached_verification(&self, signature_id: &str) -> Option<CachedSignature> {
        let cache = self.signature_cache.read().await;
        cache.get(signature_id).cloned()
    }

    /// Load signing key from configuration
    async fn load_signing_key(&self, key_config: &crate::config::SigningKeyConfig) -> Result<SigningKey> {
        // TODO: Load actual key from file/HSM/KMS
        warn!("Key loading not fully implemented, using placeholder");

        Ok(SigningKey {
            key_id: key_config.key_id.clone(),
            algorithm: key_config.algorithm.clone(),
            private_key: vec![], // Placeholder
            certificate: None,
            created_at: chrono::Utc::now(),
            expires_at: None,
        })
    }

    /// Load verification key from configuration
    async fn load_verification_key(&self, key_config: &crate::config::VerificationKeyConfig) -> Result<VerificationKey> {
        // TODO: Load actual key from file/URL
        warn!("Key loading not fully implemented, using placeholder");

        Ok(VerificationKey {
            key_id: key_config.key_id.clone(),
            algorithm: key_config.algorithm.clone(),
            public_key: vec![], // Placeholder
            certificate: None,
            trusted: key_config.trusted,
            created_at: chrono::Utc::now(),
            expires_at: None,
        })
    }

    /// Load trust store from configuration
    async fn load_trust_store(&self, trust_config: &crate::config::TrustStoreConfig) -> Result<TrustStore> {
        // TODO: Load actual certificates from files/URLs
        warn!("Trust store loading not fully implemented, using placeholder");

        Ok(TrustStore {
            name: trust_config.name.clone(),
            root_certificates: vec![],
            intermediate_certificates: vec![],
            revoked_certificates: vec![],
        })
    }
}

impl Default for VerificationPolicy {
    fn default() -> Self {
        Self {
            require_signatures: false,
            required_signatures_count: 1,
            allowed_signature_formats: vec![
                SignatureFormat::Cosign,
                SignatureFormat::NotaryV2,
                SignatureFormat::Simple,
            ],
            allowed_algorithms: vec![
                SignatureAlgorithm::EcdsaP256Sha256,
                SignatureAlgorithm::RsaPssSha256,
                SignatureAlgorithm::Ed25519,
            ],
            trust_stores: vec!["default".to_string()],
            require_certificate_chain: false,
            allow_self_signed: true,
            max_signature_age_hours: Some(24 * 30), // 30 days
        }
    }
}