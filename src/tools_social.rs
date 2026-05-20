use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::linkedin::LinkedInClient;
use crate::mcp::ServerState;
use crate::tools::{
    account_name, invalid_params, opt_i64, opt_str, require_str, token_for, VALID_VISIBILITY,
};

pub async fn create_post(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let text = require_str(&args, "text")?;
    let visibility = opt_str(&args, "visibility")?.unwrap_or("PUBLIC");
    if !VALID_VISIBILITY.contains(&visibility) {
        return Err(invalid_params(format!(
            "visibility must be one of {:?}, got '{visibility}'",
            VALID_VISIBILITY
        )));
    }
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = LinkedInClient::new(state.http.clone(), token);
    let profile = client.get_userinfo().await?;
    let sub = profile
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| invalid_params("could not resolve member sub from profile"))?;
    let author_urn = format!("urn:li:person:{sub}");
    client
        .create_ugc_post(&author_urn, text, visibility)
        .await
}

pub async fn get_own_posts(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let count = opt_i64(&args, "count")?.unwrap_or(20).clamp(1, 50);
    let start = opt_i64(&args, "start")?.unwrap_or(0).max(0);
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = LinkedInClient::new(state.http.clone(), token);
    let profile = client.get_userinfo().await?;
    let sub = profile
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| invalid_params("could not resolve member sub from profile"))?;
    let author_urn = format!("urn:li:person:{sub}");
    client.get_own_posts(&author_urn, count, start).await
}

pub async fn delete_post(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let post_id = require_str(&args, "post_id")?;
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = LinkedInClient::new(state.http.clone(), token);
    client.delete_post(post_id).await?;
    Ok(json!({ "deleted": true, "post_id": post_id }))
}
