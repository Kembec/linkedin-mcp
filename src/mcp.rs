use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

use crate::auth::Credentials;

pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const SERVER_NAME: &str = "linkedin-mcp";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct Request {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(flatten)]
    pub body: ResponseBody,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ResponseBody {
    Result { result: Value },
    Error { error: RpcError },
}

pub struct ServerState {
    pub http: reqwest::Client,
    pub token_dir: PathBuf,
    pub creds: Credentials,
}

impl ServerState {
    pub fn new() -> Result<Self> {
        let token_dir = default_token_dir()?;
        std::fs::create_dir_all(&token_dir).ok();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&token_dir, std::fs::Permissions::from_mode(0o700));
        }

        let creds = crate::auth::load_credentials()?;
        let http = reqwest::Client::builder()
            .user_agent(format!("{}/{}", SERVER_NAME, SERVER_VERSION))
            .build()?;

        Ok(Self {
            http,
            token_dir,
            creds,
        })
    }
}

pub fn default_token_dir() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("could not resolve user config directory"))?;
    Ok(base.join("kembec").join("linkedin-mcp").join("tokens"))
}

pub fn error_response(id: Value, code: i64, message: impl Into<String>) -> String {
    let resp = Response {
        jsonrpc: "2.0",
        id,
        body: ResponseBody::Error {
            error: RpcError {
                code,
                message: message.into(),
                data: None,
            },
        },
    };
    serde_json::to_string(&resp).unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialization failure"}}"#
            .to_string()
    })
}

pub fn result_response(id: Value, result: Value) -> String {
    let resp = Response {
        jsonrpc: "2.0",
        id,
        body: ResponseBody::Result { result },
    };
    serde_json::to_string(&resp)
        .unwrap_or_else(|_| error_response(Value::Null, -32603, "serialization failure"))
}

pub async fn handle_line(state: Arc<ServerState>, line: &str) -> Option<String> {
    let parsed: Result<Request, _> = serde_json::from_str(line);

    let request = match parsed {
        Ok(r) => r,
        Err(e) => {
            return Some(error_response(
                Value::Null,
                -32700,
                format!("Parse error: {e}"),
            ));
        }
    };

    let id = request.id.clone();
    let is_notification = id.is_none();

    match request.method.as_str() {
        "initialize" => {
            let result = json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION
                }
            });
            Some(result_response(id.unwrap_or(Value::Null), result))
        }
        "initialized" | "notifications/initialized" => None,
        "ping" => {
            if is_notification {
                None
            } else {
                Some(result_response(id.unwrap_or(Value::Null), json!({})))
            }
        }
        "tools/list" => {
            let tools = crate::tools::tools_list();
            Some(result_response(
                id.unwrap_or(Value::Null),
                json!({ "tools": tools }),
            ))
        }
        "tools/call" => {
            let params = request.params.unwrap_or(Value::Null);
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Object(Default::default()));

            if name.is_empty() {
                return Some(error_response(
                    id.unwrap_or(Value::Null),
                    -32602,
                    "missing tool name",
                ));
            }

            match crate::tools::call(state, &name, arguments).await {
                Ok(value) => Some(result_response(
                    id.unwrap_or(Value::Null),
                    json!({
                        "content": [
                            {
                                "type": "text",
                                "text": pretty(&value)
                            }
                        ],
                        "isError": false
                    }),
                )),
                Err(e) => {
                    let kind = classify_error(&e);
                    match kind {
                        ToolErrorKind::InvalidParams => Some(error_response(
                            id.unwrap_or(Value::Null),
                            -32602,
                            e.to_string(),
                        )),
                        ToolErrorKind::Internal => Some(result_response(
                            id.unwrap_or(Value::Null),
                            json!({
                                "content": [
                                    { "type": "text", "text": e.to_string() }
                                ],
                                "isError": true
                            }),
                        )),
                    }
                }
            }
        }
        _ => {
            if is_notification {
                None
            } else {
                Some(error_response(
                    id.unwrap_or(Value::Null),
                    -32601,
                    format!("Method not found: {}", request.method),
                ))
            }
        }
    }
}

pub enum ToolErrorKind {
    InvalidParams,
    Internal,
}

fn classify_error(e: &anyhow::Error) -> ToolErrorKind {
    let msg = format!("{e}");
    if msg.starts_with("invalid_params:") {
        ToolErrorKind::InvalidParams
    } else {
        ToolErrorKind::Internal
    }
}

fn pretty(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    }
}
