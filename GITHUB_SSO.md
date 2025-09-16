# GitHub SSO Integration

This guide explains how to configure GitHub Single Sign-On (SSO) with Drift Registry using GitHub OAuth Apps.

## Prerequisites

- GitHub account with appropriate permissions
- Access to create OAuth Apps in your GitHub organization/personal account
- Admin access to Drift Registry configuration

## Step 1: Create GitHub OAuth App

### For Personal Account
1. Go to GitHub Settings: https://github.com/settings/developers
2. Click **OAuth Apps** in the left sidebar
3. Click **New OAuth App**

### For Organization
1. Go to your organization settings: `https://github.com/orgs/YOUR_ORG/settings`
2. Click **Developer settings** > **OAuth Apps**
3. Click **New OAuth App**

## Step 2: Configure OAuth App

Fill in the application details:

- **Application name**: `Drift Registry`
- **Homepage URL**: `https://your-drift-domain.com`
- **Application description**: `Container registry and gaming optimization platform`
- **Authorization callback URL**: `https://your-drift-domain.com/auth/github/callback`

For development, you may want to create a separate OAuth app with:
- **Authorization callback URL**: `http://localhost:5001/auth/github/callback`

## Step 3: Get Client Credentials

After creating the OAuth app:

1. Note the **Client ID** (publicly visible)
2. Click **Generate a new client secret**
3. **Copy the client secret immediately** - GitHub will not show it again

## Step 4: Configure Drift Registry

Update your `config.toml` file with the GitHub OAuth settings:

```toml
[auth]
mode = "basic"  # Keep existing mode, OAuth is additional
jwt_secret = "your-jwt-secret"
token_expiry_hours = 24

[auth.oauth]
enabled = true

[auth.oauth.github]
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"
redirect_uri = "https://your-drift-domain.com/auth/github/callback"
```

### Environment Variables (Recommended for Production)

For production deployments, use environment variables:

```bash
export GITHUB_CLIENT_ID="your-github-client-id"
export GITHUB_CLIENT_SECRET="your-github-client-secret"
export GITHUB_REDIRECT_URI="https://your-drift-domain.com/auth/github/callback"
```

## Step 5: Configure Organization Access (Optional)

If you want to restrict access to specific GitHub organizations:

### Option 1: OAuth App in Organization
- Create the OAuth app under your organization settings
- Users must be members of the organization to authenticate

### Option 2: Organization Approval
1. In your organization settings, go to **Third-party access**
2. Find your OAuth app and approve it
3. Set access restrictions as needed

## Step 6: Test the Integration

1. Restart Drift Registry with the new configuration
2. Navigate to the login page
3. Click "Continue with GitHub"
4. You should be redirected to GitHub for authentication
5. Authorize the application when prompted
6. After successful authentication, you'll be redirected back to Drift Registry

## User Permissions and Scopes

The GitHub integration requests the following scopes:

- `user:email`: Access to user's email addresses
- `read:user`: Access to user profile information

### Additional Scopes (Optional)

You can request additional scopes for enhanced functionality:

```toml
[auth.oauth.github]
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://your-drift-domain.com/auth/github/callback"
scopes = ["user:email", "read:user", "read:org"]  # Add organization membership
```

Common additional scopes:
- `read:org`: Read organization membership
- `repo`: Access to repositories (if integrating with GitHub repositories)
- `admin:org`: Organization administration (for advanced integrations)

## Troubleshooting

### Common Issues

1. **Invalid Redirect URI**
   - Ensure the callback URL in GitHub exactly matches your configuration
   - URLs are case-sensitive and must match exactly

2. **Application Not Approved**
   - If using organization OAuth apps, ensure the app is approved
   - Check organization third-party access policies

3. **Rate Limiting**
   - GitHub has rate limits for OAuth requests
   - Implement proper error handling and retry logic

4. **Email Privacy**
   - Users with private email settings may not provide email addresses
   - Handle cases where email is not available

### GitHub API Debugging

Enable debug logging to see GitHub API responses:

```toml
[logging]
level = "debug"
```

Check the logs for GitHub API response details and error messages.

## Security Considerations

1. **HTTPS Required**: GitHub OAuth requires HTTPS for production redirect URIs
2. **State Parameter**: Always validate the state parameter to prevent CSRF attacks
3. **Secret Management**: Store client secrets securely, never in public repositories
4. **Token Handling**: GitHub access tokens should be handled securely and not logged

## Advanced Configuration

### Custom User Mapping

Map GitHub user attributes to Drift Registry user properties:

```toml
[auth.oauth.github.user_mapping]
username = "login"          # GitHub username
email = "email"             # Primary email
name = "name"               # Display name
avatar = "avatar_url"       # Profile picture
```

### Organization-based Access Control

Restrict access based on GitHub organization membership:

```toml
[auth.oauth.github.access_control]
required_organization = "your-org-name"
allowed_teams = ["developers", "admins"]  # Optional: specific teams
```

### Webhook Integration

For real-time updates, configure GitHub webhooks:

1. In your GitHub organization/repository settings
2. Add webhook URL: `https://your-drift-domain.com/webhooks/github`
3. Select relevant events (member added/removed, team changes)

## GitHub Enterprise Support

For GitHub Enterprise Server installations:

```toml
[auth.oauth.github]
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://your-drift-domain.com/auth/github/callback"
enterprise_url = "https://github.yourcompany.com"  # Your GHE instance
api_url = "https://github.yourcompany.com/api/v3"   # GHE API endpoint
```

## Rate Limits

GitHub OAuth has the following rate limits:
- 5,000 requests per hour for authenticated requests
- 60 requests per hour for unauthenticated requests

Implement appropriate caching and rate limiting in your application.

## Support and Resources

- [GitHub OAuth Apps Documentation](https://docs.github.com/en/developers/apps/building-oauth-apps)
- [GitHub API Documentation](https://docs.github.com/en/rest)
- [OAuth 2.0 RFC](https://tools.ietf.org/html/rfc6749)
- Drift Registry logs and error messages