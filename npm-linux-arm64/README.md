# linkedin-mcp

[![npm](https://img.shields.io/npm/v/@kembec/linkedin-mcp)](https://www.npmjs.com/package/@kembec/linkedin-mcp)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

LinkedIn MCP server — read your profile, publish posts, and manage your LinkedIn presence from Cursor, Claude Desktop, or any MCP-compatible AI client. Single Rust binary, no runtime required.

## Step 1 — Create a LinkedIn Developer App

1. Go to [LinkedIn Developer Portal](https://www.linkedin.com/developers/apps) and click **Create app**
2. Fill in app name, associate it with a LinkedIn Page, and accept terms
3. Under **Auth** tab → **OAuth 2.0 settings**, add this redirect URL:
   ```
   http://127.0.0.1:9876/callback
   ```
4. Under **Products** tab, request **Sign In with LinkedIn using OpenID Connect** (approved instantly)
5. Go back to **Auth** tab and copy your **Client ID** and **Client Secret**

That's it — you now have everything needed to authenticate.

## Step 2 — Install

```bash
npm install -g @kembec/linkedin-mcp
```

Or run without installing:

```bash
npx -y @kembec/linkedin-mcp auth
```

## Step 3 — Authenticate

```bash
export LINKEDIN_CLIENT_ID="your-client-id"
export LINKEDIN_CLIENT_SECRET="your-client-secret"
linkedin-mcp auth
```

Your browser opens automatically. Log in to LinkedIn, click **Allow**, and the token is saved. Done.

To use multiple accounts:

```bash
linkedin-mcp auth work
linkedin-mcp auth personal
```

Tokens are stored at `~/.config/kembec/linkedin-mcp/tokens/` with permissions `0600`. Credentials are never written to disk.

## Step 4 — Configure your AI client

### Cursor

Add to `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "linkedin": {
      "command": "npx",
      "args": ["-y", "@kembec/linkedin-mcp"],
      "env": {
        "LINKEDIN_CLIENT_ID": "your-client-id",
        "LINKEDIN_CLIENT_SECRET": "your-client-secret"
      }
    }
  }
}
```

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "linkedin": {
      "command": "npx",
      "args": ["-y", "@kembec/linkedin-mcp"],
      "env": {
        "LINKEDIN_CLIENT_ID": "your-client-id",
        "LINKEDIN_CLIENT_SECRET": "your-client-secret"
      }
    }
  }
}
```

### Codex CLI

Add to `~/.codex/config.toml`:

```toml
[mcp_servers.linkedin]
command = "npx"
args = ["-y", "@kembec/linkedin-mcp"]

[mcp_servers.linkedin.env]
LINKEDIN_CLIENT_ID = "your-client-id"
LINKEDIN_CLIENT_SECRET = "your-client-secret"
```

## Available tools

### Profile & posts (standard access — no partner approval needed)

| Tool | Description |
|------|-------------|
| `get-profile` | Your profile: name, email, profile picture URL |
| `create-post` | Publish a text post. Params: `text` (required), `visibility` (`PUBLIC` or `CONNECTIONS`, default `PUBLIC`) |
| `get-own-posts` | List your recent posts. Params: `count` (default 10), `start` (default 0) |
| `delete-post` | Delete a post by post ID or full URN |
| `get-company` | Organization details by `org_id` |
| `manage-accounts` | List or remove stored OAuth tokens. Params: `action` (`list` or `remove`), `account` (for remove) |

### Tools that require LinkedIn Partner approval

These tools are implemented and return a descriptive error if your app lacks the required scope. Apply at [LinkedIn Developer — Product catalog](https://developer.linkedin.com/product-catalog).

| Tool | Required scope | Description |
|------|----------------|-------------|
| `get-connections` | `r_network` | List 1st-degree connections |
| `search-jobs` | `r_jobs` | Search job postings by keywords and location |
| `send-message` | `w_messages` | Send a direct message to a member URN |

### Tools not available in the standard API

| Tool | Notes |
|------|-------|
| `search-people` | Requires Recruiter, Talent Hub, or Sales Navigator — not part of the standard member API |

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `LINKEDIN_CLIENT_ID` | Yes | OAuth app client ID |
| `LINKEDIN_CLIENT_SECRET` | Yes | OAuth app client secret |
| `LINKEDIN_OAUTH_PORT` | No | Fixed port for the local OAuth callback server (default: dynamic). Set to `9876` if you registered `http://127.0.0.1:9876/callback` |
| `LINKEDIN_REDIRECT_URI` | No | Override the full redirect URI (advanced use) |

## Building from source

```bash
git clone https://github.com/Kembec/linkedin-mcp.git
cd linkedin-mcp
cargo build --release
export LINKEDIN_CLIENT_ID="your-client-id"
export LINKEDIN_CLIENT_SECRET="your-client-secret"
./target/release/linkedin-mcp auth
```

## License

MIT — see [LICENSE](LICENSE).
