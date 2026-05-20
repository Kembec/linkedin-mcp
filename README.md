# linkedin-mcp

[![npm](https://img.shields.io/npm/v/@kembec/linkedin-mcp)](https://www.npmjs.com/package/@kembec/linkedin-mcp)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

LinkedIn MCP server — read your profile, publish posts, and explore your network. Single binary, no runtime required.

## Prerequisites

- A [LinkedIn Developer App](https://www.linkedin.com/developers/apps) with OAuth 2.0 enabled
- Add `http://127.0.0.1` as an authorized redirect URL (any port; the CLI binds a dynamic local port)
- Environment variables:
  - `LINKEDIN_CLIENT_ID` — from your app credentials
  - `LINKEDIN_CLIENT_SECRET` — from your app credentials

Credentials are read from the environment only and are never written to disk. OAuth tokens are stored under `~/.config/kembec/linkedin-mcp/tokens/` with file mode `0600`.

## Installation

```bash
npm install -g @kembec/linkedin-mcp
```

Or run without installing:

```bash
npx @kembec/linkedin-mcp
```

## Authentication

```bash
export LINKEDIN_CLIENT_ID="your-client-id"
export LINKEDIN_CLIENT_SECRET="your-client-secret"
linkedin-mcp auth
linkedin-mcp auth my-work-account
```

The `auth` command opens a browser for LinkedIn OAuth and saves the token locally.

## Configuration

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

## Tools — Standard access

| Tool | Description |
|------|-------------|
| `get-profile` | Authenticated member profile (`sub`, `name`, `email`) |
| `create-post` | Publish a text post (`text`, optional `visibility`) |
| `get-own-posts` | List your UGC posts (`count`, `start`) |
| `delete-post` | Delete a post by id or URN |
| `get-company` | Organization details by `org_id` |
| `manage-accounts` | List, add, or remove stored OAuth accounts |

## Tools — Requires LinkedIn Partner approval

These tools call the API but return a clear error if your app lacks the required scope:

| Tool | Scope | Description |
|------|-------|-------------|
| `get-connections` | `r_network` | List 1st-degree connections |
| `search-jobs` | `r_jobs` | Search job postings by keywords |
| `send-message` | `w_messages` | Send a message to a member URN |

Apply for partner products at [LinkedIn Developer — Product catalog](https://developer.linkedin.com/product-catalog).

## Tools — Not available in standard API

| Tool | Notes |
|------|-------|
| `search-people` | People search requires Recruiter, Talent Hub, or Sales Navigator APIs — not exposed in the standard member API |

## Building from source

```bash
git clone https://github.com/Kembec/linkedin-mcp.git
cd linkedin-mcp
cargo build --release
./target/release/linkedin-mcp auth
```

## License

MIT — see [LICENSE](LICENSE).
