# Google SSO Integration

This guide explains how to configure Google Single Sign-On (SSO) with Drift Registry using Google OAuth 2.0.

## Prerequisites

- Google account with Google Cloud Console access
- Ability to create OAuth 2.0 credentials
- Admin access to Drift Registry configuration

## Step 1: Create Google Cloud Project

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one:
   - Click the project dropdown at the top
   - Click **New Project**
   - Enter project name: `Drift Registry` (or your preferred name)
   - Click **Create**

## Step 2: Enable Google+ API

1. In the Google Cloud Console, go to **APIs & Services** > **Library**
2. Search for "Google+ API" or "People API"
3. Click on the API and click **Enable**
4. Also enable "Google OAuth2 API" if available

## Step 3: Configure OAuth Consent Screen

1. Go to **APIs & Services** > **OAuth consent screen**
2. Choose **User Type**:
   - **Internal**: Only users in your Google Workspace organization
   - **External**: Any Google account user
3. Fill in the required information:
   - **App name**: `Drift Registry`
   - **User support email**: Your email address
   - **App logo**: Upload your Drift logo (optional)
   - **App domain**: `your-drift-domain.com`
   - **Authorized domains**: Add `your-drift-domain.com`
   - **Developer contact information**: Your email

4. **Scopes**: Add the following scopes:
   - `openid`
   - `profile`
   - `email`

5. **Test users** (for External apps in testing mode):
   - Add email addresses of users who can test the integration

## Step 4: Create OAuth 2.0 Credentials

1. Go to **APIs & Services** > **Credentials**
2. Click **Create Credentials** > **OAuth client ID**
3. Choose **Application type**: **Web application**
4. Configure the OAuth client:
   - **Name**: `Drift Registry Web Client`
   - **Authorized JavaScript origins**:
     - `https://your-drift-domain.com`
     - `http://localhost:5001` (for development)
   - **Authorized redirect URIs**:
     - `https://your-drift-domain.com/auth/google/callback`
     - `http://localhost:5001/auth/google/callback` (for development)

5. Click **Create**
6. **Copy the Client ID and Client Secret** - you'll need these for configuration

## Step 5: Configure Drift Registry

Update your `config.toml` file with the Google OAuth settings:

```toml
[auth]
mode = "basic"  # Keep existing mode, OAuth is additional
jwt_secret = "your-jwt-secret"
token_expiry_hours = 24

[auth.oauth]
enabled = true

[auth.oauth.google]
client_id = "your-google-client-id.apps.googleusercontent.com"
client_secret = "your-google-client-secret"
redirect_uri = "https://your-drift-domain.com/auth/google/callback"
```

### Environment Variables (Recommended for Production)

For production deployments, use environment variables:

```bash
export GOOGLE_CLIENT_ID="your-google-client-id.apps.googleusercontent.com"
export GOOGLE_CLIENT_SECRET="your-google-client-secret"
export GOOGLE_REDIRECT_URI="https://your-drift-domain.com/auth/google/callback"
```

## Step 6: Domain Verification (For External Apps)

If you're using External user type and want to publish your app:

1. Go to **APIs & Services** > **Domain verification**
2. Add and verify your domain
3. This allows you to remove the "unverified app" warning

## Step 7: Test the Integration

1. Restart Drift Registry with the new configuration
2. Navigate to the login page
3. Click "Continue with Google"
4. You should be redirected to Google for authentication
5. Sign in with your Google account
6. Grant permissions when prompted
7. After successful authentication, you'll be redirected back to Drift Registry

## Google Workspace Integration

### Domain Restriction

To restrict access to users from specific Google Workspace domains:

```toml
[auth.oauth.google]
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://your-drift-domain.com/auth/google/callback"
allowed_domains = ["yourcompany.com", "subsidiary.com"]
```

### Admin SDK Integration

For advanced Google Workspace integration (requires additional setup):

1. Enable the Admin SDK API
2. Create a service account
3. Configure domain-wide delegation
4. Add additional scopes for directory access

## Troubleshooting

### Common Issues

1. **Redirect URI Mismatch**
   - Ensure redirect URIs in Google Cloud Console exactly match your configuration
   - URIs are case-sensitive and must match exactly including protocol (http/https)

2. **Unverified App Warning**
   - For External apps, users may see an "unverified app" warning
   - Add test users to bypass this during development
   - Complete the verification process for production

3. **API Not Enabled**
   - Ensure Google+ API or People API is enabled for your project
   - Check the APIs & Services > Library section

4. **Quota Exceeded**
   - Google APIs have usage quotas
   - Check APIs & Services > Quotas for current usage

### Google API Debugging

Enable debug logging to see Google API responses:

```toml
[logging]
level = "debug"
```

Check the logs for Google API response details and error messages.

## Security Considerations

1. **HTTPS Required**: Google OAuth requires HTTPS for production redirect URIs
2. **State Parameter**: Always validate the state parameter to prevent CSRF attacks
3. **Token Storage**: Handle Google access tokens securely
4. **Scope Limitation**: Only request necessary scopes

## Advanced Configuration

### Custom User Mapping

Map Google user attributes to Drift Registry user properties:

```toml
[auth.oauth.google.user_mapping]
username = "email"          # Use email as username
email = "email"             # Primary email
name = "name"               # Display name
picture = "picture"         # Profile picture URL
locale = "locale"           # User's locale
```

### Additional Scopes

Request additional scopes for enhanced functionality:

```toml
[auth.oauth.google]
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://your-drift-domain.com/auth/google/callback"
scopes = ["openid", "profile", "email", "https://www.googleapis.com/auth/admin.directory.user.readonly"]
```

Common additional scopes:
- `https://www.googleapis.com/auth/admin.directory.user.readonly`: Read user directory information
- `https://www.googleapis.com/auth/admin.directory.group.readonly`: Read group membership
- `https://www.googleapis.com/auth/calendar.readonly`: Read calendar information

### Google Workspace Admin Controls

Google Workspace administrators can control access to OAuth apps:

1. **Admin Console** > **Security** > **API controls**
2. **App access control**: Configure which apps can access Google Workspace data
3. **OAuth app verification**: Control which OAuth apps are allowed

## Rate Limits and Quotas

Google APIs have the following limits:
- **Requests per 100 seconds per user**: 100
- **Requests per 100 seconds**: 1,000

Implement appropriate rate limiting and error handling in your application.

## Production Deployment

### App Verification

For production use with External user type:

1. Complete OAuth consent screen verification
2. Provide privacy policy and terms of service URLs
3. Submit app for verification if requesting sensitive scopes
4. The verification process can take several days

### Monitoring

Set up monitoring for Google OAuth integration:
- Monitor authentication success/failure rates
- Track API quota usage
- Set up alerts for authentication errors

## Support and Resources

- [Google OAuth 2.0 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [Google Cloud Console](https://console.cloud.google.com/)
- [OAuth 2.0 Playground](https://developers.google.com/oauthplayground/) (for testing)
- [Google Workspace Admin Help](https://support.google.com/a/answer/7281227)
- Drift Registry logs and error messages

## Migration from Google+ API

If you're migrating from the deprecated Google+ API:
- Update to use People API or Google OAuth2 API
- Update scope requests from `https://www.googleapis.com/auth/plus.login` to `openid profile email`
- Test thoroughly as response formats may differ