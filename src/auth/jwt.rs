use crate::auth::{User, AuthToken};
use anyhow::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

pub fn generate_token(secret: &str, user: &User, expires_in: u64) -> Result<String> {
    let now = chrono::Utc::now().timestamp() as u64;
    let exp = now + expires_in;

    let claims = AuthToken {
        user: user.clone(),
        exp,
        iat: now,
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_ref());

    let token = encode(&header, &claims, &key)?;
    Ok(token)
}

pub fn validate_token(secret: &str, token: &str) -> Result<Option<User>> {
    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    match decode::<AuthToken>(token, &key, &validation) {
        Ok(token_data) => {
            let now = chrono::Utc::now().timestamp() as u64;
            if token_data.claims.exp > now {
                Ok(Some(token_data.claims.user))
            } else {
                Ok(None) // Token expired
            }
        }
        Err(_) => Ok(None), // Invalid token
    }
}