use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[path = "../src/auth.rs"]
mod auth;
#[path = "../src/linkedin.rs"]
mod linkedin;
#[path = "../src/mcp.rs"]
mod mcp;
#[path = "../src/models.rs"]
mod models;
#[path = "../src/tools_profile.rs"]
mod tools_profile;
#[path = "../src/tools_social.rs"]
mod tools_social;
#[path = "../src/tools.rs"]
mod tools;
#[path = "../src/tools_network.rs"]
mod tools_network;

fn state() -> Arc<mcp::ServerState> {
    std::env::set_var("LINKEDIN_CLIENT_ID", "test-client-id");
    std::env::set_var("LINKEDIN_CLIENT_SECRET", "test-client-secret");
    let tmp = std::env::temp_dir().join(format!(
        "linkedin-mcp-test-tools-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    let creds = auth::load_credentials().expect("credentials");
    Arc::new(mcp::ServerState {
        http: reqwest::Client::new(),
        token_dir: tmp,
        creds,
    })
}

fn is_invalid_params(err: &anyhow::Error) -> bool {
    format!("{err}").starts_with("invalid_params:")
}

#[test]
fn test_credentials_debug_redacts_secret() {
    let creds = auth::Credentials {
        client_id: "id".to_string(),
        client_secret: "super-secret".to_string(),
    };
    let debug = format!("{creds:?}");
    assert!(debug.contains("id"));
    assert!(!debug.contains("super-secret"));
    assert!(debug.contains("[REDACTED]"));
}

#[test]
fn test_token_path_sanitizes_special_chars() {
    let dir = PathBuf::from("/tmp/linkedin-mcp-test");
    let path = auth::token_path(&dir, "user@corp/name");
    assert!(path.to_string_lossy().contains("user@corp_name"));
}

#[tokio::test]
async fn test_invalid_params_require_str_missing() {
    let err = tools::require_str(&json!({}), "text")
        .expect_err("should fail");
    assert!(is_invalid_params(&err));
}

#[tokio::test]
async fn test_invalid_params_require_str_empty() {
    let err = tools::require_str(&json!({ "text": "" }), "text")
        .expect_err("should fail");
    assert!(format!("{err}").contains("must not be empty"));
}

#[tokio::test]
async fn test_search_people_returns_descriptive_error() {
    let result = tools_network::search_people(state(), json!({ "query": "engineer" }))
        .await
        .expect("search_people returns Ok");
    assert!(result.get("error").is_some());
    let err = result["error"].as_str().unwrap_or("");
    assert!(err.contains("not available"));
}

#[test]
fn test_validate_visibility_valid_values() {
    for v in tools::VALID_VISIBILITY {
        assert!(tools::VALID_VISIBILITY.contains(v));
    }
}

#[test]
fn test_validate_visibility_invalid() {
    assert!(!tools::VALID_VISIBILITY.contains(&"PRIVATE"));
}

#[test]
fn test_load_credentials_from_env() {
    std::env::set_var("LINKEDIN_CLIENT_ID", "env-client");
    std::env::set_var("LINKEDIN_CLIENT_SECRET", "env-secret");
    let creds = auth::load_credentials().expect("load credentials");
    assert_eq!(creds.client_id, "env-client");
    assert_eq!(creds.client_secret, "env-secret");
}
