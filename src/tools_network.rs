use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::linkedin::LinkedInClient;
use crate::mcp::ServerState;
use crate::tools::{
    account_name, map_partner_error, opt_i64, require_str, token_for,
};

pub async fn get_connections(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let start = opt_i64(&args, "start")?.unwrap_or(0).max(0);
    let count = opt_i64(&args, "count")?.unwrap_or(20).clamp(1, 50);
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = LinkedInClient::new(state.http.clone(), token);
    client
        .get_connections(start, count)
        .await
        .map_err(|e| map_partner_error(e, "r_network"))
}

pub async fn get_company(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let org_id = require_str(&args, "org_id")?;
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = LinkedInClient::new(state.http.clone(), token);
    client.get_organization(org_id).await
}

pub async fn search_jobs(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let keywords = require_str(&args, "keywords")?;
    let count = opt_i64(&args, "count")?.unwrap_or(10).clamp(1, 25);
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = LinkedInClient::new(state.http.clone(), token);
    client
        .search_jobs(keywords, count)
        .await
        .map_err(|e| map_partner_error(e, "r_jobs"))
}

pub async fn send_message(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let recipient_urn = require_str(&args, "recipient_urn")?;
    let text = require_str(&args, "text")?;
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = LinkedInClient::new(state.http.clone(), token);
    client
        .send_message(recipient_urn, text)
        .await
        .map_err(|e| map_partner_error(e, "w_messages"))
}

pub async fn search_people(_state: Arc<ServerState>, args: Value) -> Result<Value> {
    let _query = require_str(&args, "query")?;
    Ok(json!({
        "error": "People search is not available in the LinkedIn standard API.",
        "details": "Search API requires LinkedIn Recruiter, Talent Hub, or Sales Navigator API access. See https://developer.linkedin.com/product-catalog"
    }))
}
