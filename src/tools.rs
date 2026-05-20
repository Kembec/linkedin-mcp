use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::auth;
use crate::mcp::ServerState;
use crate::tools_network::{get_company, get_connections, search_jobs, search_people, send_message};
use crate::tools_profile::get_profile;
use crate::tools_social::{create_post, delete_post, get_own_posts};

pub const DEFAULT_ACCOUNT: &str = "default";
pub const VALID_VISIBILITY: &[&str] = &["PUBLIC", "CONNECTIONS", "LOGGED_IN"];

pub fn tools_list() -> Value {
    json!([
        {
            "name": "get-profile",
            "description": "Get the authenticated member profile (name, email, sub).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "account": { "type": "string", "description": "Account name (defaults to 'default')." }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "create-post",
            "description": "Publish a text post to LinkedIn.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Post body text." },
                    "visibility": {
                        "type": "string",
                        "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"],
                        "description": "Who can see the post (default PUBLIC)."
                    },
                    "account": { "type": "string" }
                },
                "required": ["text"],
                "additionalProperties": false
            }
        },
        {
            "name": "get-own-posts",
            "description": "List posts authored by the authenticated member.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "count": { "type": "integer", "minimum": 1, "maximum": 50, "default": 20 },
                    "start": { "type": "integer", "minimum": 0, "default": 0 },
                    "account": { "type": "string" }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "delete-post",
            "description": "Delete a UGC post by id or URN.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "post_id": { "type": "string" },
                    "account": { "type": "string" }
                },
                "required": ["post_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "get-company",
            "description": "Fetch organization details by numeric organization id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "org_id": { "type": "string" },
                    "account": { "type": "string" }
                },
                "required": ["org_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "get-connections",
            "description": "List 1st-degree connections (requires LinkedIn Partner approval for r_network).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "start": { "type": "integer", "minimum": 0, "default": 0 },
                    "count": { "type": "integer", "minimum": 1, "maximum": 50, "default": 20 },
                    "account": { "type": "string" }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "search-jobs",
            "description": "Search job postings by keywords (requires r_jobs scope approval).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "keywords": { "type": "string" },
                    "count": { "type": "integer", "minimum": 1, "maximum": 25, "default": 10 },
                    "account": { "type": "string" }
                },
                "required": ["keywords"],
                "additionalProperties": false
            }
        },
        {
            "name": "send-message",
            "description": "Send a message to a member URN (requires Messaging API approval).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "recipient_urn": { "type": "string" },
                    "text": { "type": "string" },
                    "account": { "type": "string" }
                },
                "required": ["recipient_urn", "text"],
                "additionalProperties": false
            }
        },
        {
            "name": "search-people",
            "description": "People search is not available in the standard LinkedIn API.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"],
                "additionalProperties": false
            }
        },
        {
            "name": "manage-accounts",
            "description": "List, add, or remove stored OAuth accounts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["list", "add", "remove"] },
                    "account_name": { "type": "string" }
                },
                "required": ["action"],
                "additionalProperties": false
            }
        }
    ])
}

pub async fn call(state: Arc<ServerState>, name: &str, args: Value) -> Result<Value> {
    match name {
        "get-profile" => get_profile(state, args).await,
        "create-post" => create_post(state, args).await,
        "get-own-posts" => get_own_posts(state, args).await,
        "delete-post" => delete_post(state, args).await,
        "get-company" => get_company(state, args).await,
        "get-connections" => get_connections(state, args).await,
        "search-jobs" => search_jobs(state, args).await,
        "send-message" => send_message(state, args).await,
        "search-people" => search_people(state, args).await,
        "manage-accounts" => manage_accounts(state, args).await,
        _ => Err(invalid_params(format!("unknown tool: {name}"))),
    }
}

pub fn invalid_params(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("invalid_params: {}", msg.into())
}

pub fn require_str<'a>(args: &'a Value, field: &str) -> Result<&'a str> {
    let v = args
        .get(field)
        .ok_or_else(|| invalid_params(format!("missing required field '{field}'")))?;
    let s = v
        .as_str()
        .ok_or_else(|| invalid_params(format!("field '{field}' must be a string")))?;
    if s.is_empty() {
        return Err(invalid_params(format!("field '{field}' must not be empty")));
    }
    Ok(s)
}

pub fn opt_str<'a>(args: &'a Value, field: &str) -> Result<Option<&'a str>> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if !s.is_empty() => Ok(Some(s.as_str())),
        Some(Value::String(_)) => Ok(None),
        _ => Err(invalid_params(format!("field '{field}' must be a string"))),
    }
}

pub fn opt_i64(args: &Value, field: &str) -> Result<Option<i64>> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n
            .as_i64()
            .ok_or_else(|| invalid_params(format!("field '{field}' must be an integer")))
            .map(Some),
        _ => Err(invalid_params(format!("field '{field}' must be an integer"))),
    }
}

pub fn account_name<'a>(args: &'a Value) -> &'a str {
    args.get("account")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_ACCOUNT)
}

pub async fn token_for(state: &ServerState, account: &str) -> Result<String> {
    auth::get_token(
        &state.http,
        &state.token_dir,
        &state.creds,
        account,
    )
    .await
}

pub fn partner_access_hint(scope: &str) -> String {
    format!(
        "LinkedIn returned access denied. The '{scope}' scope requires LinkedIn Partner Program approval. \
See https://developer.linkedin.com/product-catalog"
    )
}

pub fn map_partner_error(e: anyhow::Error, scope: &str) -> anyhow::Error {
    let msg = format!("{e}");
    if msg.contains("403") || msg.contains("401") {
        anyhow::anyhow!("{}", partner_access_hint(scope))
    } else {
        e
    }
}

async fn manage_accounts(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let action = require_str(&args, "action")?;
    match action {
        "list" => {
            let accounts = auth::list_accounts(&state.token_dir);
            Ok(json!({ "accounts": accounts }))
        }
        "add" => {
            let account = args
                .get("account_name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(DEFAULT_ACCOUNT);
            let token = auth::interactive_login(&state.http, &state.creds).await?;
            auth::save_token(&state.token_dir, account, &token)?;
            Ok(json!({
                "ok": true,
                "account": account,
                "scopes": token.scope
            }))
        }
        "remove" => {
            let account = args
                .get("account_name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| invalid_params("'remove' requires 'account_name'"))?;
            auth::remove_account(&state.token_dir, account)?;
            Ok(json!({ "ok": true, "removed": account }))
        }
        other => Err(invalid_params(format!(
            "action must be one of list/add/remove, got '{other}'"
        ))),
    }
}
