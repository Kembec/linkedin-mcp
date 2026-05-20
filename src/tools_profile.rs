use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;

use crate::linkedin::LinkedInClient;
use crate::mcp::ServerState;
use crate::tools::{account_name, token_for};

pub async fn get_profile(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = LinkedInClient::new(state.http.clone(), token);
    client.get_userinfo().await
}
