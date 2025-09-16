# Azure AD SSO Integration

This guide explains how to configure Azure Active Directory (Azure AD) Single Sign-On (SSO) with Drift Registry.

## Prerequisites

- Azure AD tenant access
- Application registration permissions
- Admin access to Drift Registry configuration

## Step 1: Create Azure AD App Registration

1. Sign in to the [Azure Portal](https://portal.azure.com)
2. Navigate to **Azure Active Directory** > **App registrations**
3. Click **New registration**
4. Configure the application:
   - **Name**: `Drift Registry`
   - **Supported account types**: Select based on your needs (usually "Accounts in this organizational directory only")
   - **Redirect URI**:
     - Type: Web
     - URL: `https://your-drift-domain.com/auth/azure/callback`

## Step 2: Configure Application Settings

### Authentication
1. In your app registration, go to **Authentication**
2. Add additional redirect URIs if needed:
   - `http://localhost:5001/auth/azure/callback` (for development)
3. Under **Implicit grant and hybrid flows**, enable:
   - ✅ ID tokens
4. Set **Logout URL**: `https://your-drift-domain.com/auth/logout`

### API Permissions
1. Go to **API permissions**
2. Add the following Microsoft Graph permissions:
   - `openid` (delegated)
   - `profile` (delegated)
   - `email` (delegated)
   - `User.Read` (delegated)
3. Grant admin consent for these permissions

### Certificates & Secrets
1. Go to **Certificates & secrets**
2. Create a new client secret:
   - Description: `Drift Registry Secret`
   - Expires: Choose appropriate duration
3. **Copy the secret value immediately** - you won't be able to see it again

## Step 3: Configure Drift Registry

Update your `config.toml` file with the Azure AD settings:

```toml
[auth]
mode = "basic"  # Keep existing mode, OAuth is additional
jwt_secret = "your-jwt-secret"
token_expiry_hours = 24

[auth.oauth]
enabled = true

[auth.oauth.azure]
tenant_id = "your-tenant-id"  # Found in Azure AD Overview
client_id = "your-application-id"  # Found in app registration Overview
client_secret = "your-client-secret"  # The secret you created
redirect_uri = "https://your-drift-domain.com/auth/azure/callback"
```

### Environment Variables (Alternative)

For production deployments, use environment variables instead of storing secrets in config files:

```bash
export AZURE_TENANT_ID="your-tenant-id"
export AZURE_CLIENT_ID="your-application-id"
export AZURE_CLIENT_SECRET="your-client-secret"
export AZURE_REDIRECT_URI="https://your-drift-domain.com/auth/azure/callback"
```

## Step 4: User Assignment (Optional)

To restrict access to specific users:

1. In Azure AD, go to **Enterprise applications**
2. Find your "Drift Registry" application
3. Go to **Properties**
4. Set **User assignment required?** to **Yes**
5. Go to **Users and groups** to assign specific users/groups

## Step 5: Test the Integration

1. Restart Drift Registry with the new configuration
2. Navigate to the login page
3. Click "Continue with Azure AD"
4. You should be redirected to Azure AD for authentication
5. After successful authentication, you'll be redirected back to Drift Registry

## Troubleshooting

### Common Issues

1. **Invalid Redirect URI**
   - Ensure the redirect URI in Azure AD exactly matches your configuration
   - Include both production and development URLs if needed

2. **Missing Permissions**
   - Verify all required API permissions are granted
   - Ensure admin consent is provided

3. **Token Validation Errors**
   - Check that the tenant ID is correct
   - Verify the client ID matches the app registration

4. **User Not Found**
   - If using user assignment, ensure the user is assigned to the application
   - Check that the user has appropriate licenses

### Logs and Debugging

Enable debug logging in Drift Registry:

```toml
[logging]
level = "debug"
```

Look for Azure AD-related log entries to diagnose authentication issues.

## Security Considerations

1. **Use HTTPS**: Always use HTTPS in production for OAuth flows
2. **Rotate Secrets**: Regularly rotate client secrets
3. **Principle of Least Privilege**: Only request necessary permissions
4. **Monitor Access**: Use Azure AD sign-in logs to monitor authentication

## Advanced Configuration

### Custom Claims Mapping

To map Azure AD attributes to Drift Registry user properties, configure claim mappings:

```toml
[auth.oauth.azure.claims]
username = "preferred_username"
email = "email"
name = "name"
groups = "groups"  # For role-based access control
```

### Conditional Access

Azure AD Conditional Access policies can be applied to control access to Drift Registry based on:
- User location
- Device compliance
- Risk level
- Multi-factor authentication requirements

Configure these policies in Azure AD under **Security** > **Conditional Access**.

## Support

For issues with Azure AD configuration, consult:
- [Azure AD documentation](https://docs.microsoft.com/en-us/azure/active-directory/)
- [OAuth 2.0 and OpenID Connect protocols](https://docs.microsoft.com/en-us/azure/active-directory/develop/active-directory-v2-protocols)
- Drift Registry logs and error messages