use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

pub const AUTH_URL: &str = "https://www.linkedin.com/oauth/v2/authorization";
pub const TOKEN_URL: &str = "https://www.linkedin.com/oauth/v2/accessToken";
pub const SCOPE: &str = "openid profile email w_member_social r_organization_social r_network r_1st_connections r_jobs w_messages";
pub const DEFAULT_TOKEN_TTL_SECS: i64 = 5_184_000;

#[derive(Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expiry: i64,
    pub scope: Option<String>,
}

impl fmt::Debug for StoredToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoredToken")
            .field("access_token", &"[REDACTED]")
            .field("refresh_token", &self.refresh_token.as_ref().map(|_| "[REDACTED]"))
            .field("expiry", &self.expiry)
            .field("scope", &self.scope)
            .finish()
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    #[serde(default)]
    scope: Option<String>,
}

pub fn load_credentials() -> Result<Credentials> {
    let client_id = std::env::var("LINKEDIN_CLIENT_ID")
        .map_err(|_| anyhow!("LINKEDIN_CLIENT_ID env var is not set"))?;
    let client_secret = std::env::var("LINKEDIN_CLIENT_SECRET")
        .map_err(|_| anyhow!("LINKEDIN_CLIENT_SECRET env var is not set"))?;
    if client_id.trim().is_empty() {
        return Err(anyhow!("LINKEDIN_CLIENT_ID must not be empty"));
    }
    if client_secret.trim().is_empty() {
        return Err(anyhow!("LINKEDIN_CLIENT_SECRET must not be empty"));
    }
    Ok(Credentials {
        client_id: client_id.trim().to_string(),
        client_secret: client_secret.trim().to_string(),
    })
}

pub fn token_path(token_dir: &Path, account: &str) -> PathBuf {
    let safe = account
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    token_dir.join(format!("{safe}.json"))
}

pub fn load_token(token_dir: &Path, account: &str) -> Result<Option<StoredToken>> {
    let path = token_path(token_dir, account);
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read token file {}", path.display()))?;
    let token: StoredToken = serde_json::from_str(&raw)
        .with_context(|| format!("parse token JSON {}", path.display()))?;
    Ok(Some(token))
}

pub fn save_token(token_dir: &Path, account: &str, token: &StoredToken) -> Result<()> {
    std::fs::create_dir_all(token_dir).ok();
    let path = token_path(token_dir, account);
    let raw = serde_json::to_string_pretty(token)?;
    std::fs::write(&path, raw).with_context(|| format!("write token file {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn list_accounts(token_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(token_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                out.push(stem.to_string());
            }
        }
    }
    out.sort();
    out
}

pub fn remove_account(token_dir: &Path, account: &str) -> Result<()> {
    let path = token_path(token_dir, account);
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("remove token file {}", path.display()))?;
    }
    Ok(())
}

pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

async fn refresh_access_token(
    http: &reqwest::Client,
    creds: &Credentials,
    refresh_token: &str,
) -> Result<TokenResponse> {
    let form = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", creds.client_id.as_str()),
        ("client_secret", creds.client_secret.as_str()),
    ];
    let resp = http
        .post(TOKEN_URL)
        .form(&form)
        .send()
        .await
        .context("token refresh request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("token refresh failed ({status}): {body}"));
    }
    let parsed: TokenResponse = resp.json().await.context("parse refresh response")?;
    Ok(parsed)
}

async fn exchange_code(
    http: &reqwest::Client,
    creds: &Credentials,
    code: &str,
    redirect_uri: &str,
) -> Result<TokenResponse> {
    let form = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", creds.client_id.as_str()),
        ("client_secret", creds.client_secret.as_str()),
    ];
    let resp = http
        .post(TOKEN_URL)
        .form(&form)
        .send()
        .await
        .context("token exchange request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("token exchange failed ({status}): {body}"));
    }
    let parsed: TokenResponse = resp.json().await.context("parse token response")?;
    Ok(parsed)
}

fn find_free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

pub async fn interactive_login(
    http: &reqwest::Client,
    creds: &Credentials,
) -> Result<StoredToken> {
    let port = find_free_port()?;
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let state_param = uuid::Uuid::new_v4().to_string();

    let mut authorize = url::Url::parse(AUTH_URL)?;
    authorize
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &creds.client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("state", &state_param)
        .append_pair("scope", SCOPE);

    let url_string = authorize.to_string();
    eprintln!("Open this URL in a browser to authorize linkedin-mcp:");
    eprintln!("{url_string}");
    let _ = open::that(url_string);

    let (tx, rx) = mpsc::channel::<Result<String, String>>();
    let expected_state = state_param.clone();
    let bind_addr = format!("127.0.0.1:{port}");
    std::thread::spawn(move || {
        let server = match tiny_http::Server::http(&bind_addr) {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(Err(format!("failed to start local server: {e}")));
                return;
            }
        };
        for request in server.incoming_requests() {
            let url_str = format!("http://localhost{}", request.url());
            let parsed = url::Url::parse(&url_str);
            let mut code = None;
            let mut state_in = None;
            let mut error = None;
            if let Ok(u) = parsed {
                for (k, v) in u.query_pairs() {
                    match k.as_ref() {
                        "code" => code = Some(v.into_owned()),
                        "state" => state_in = Some(v.into_owned()),
                        "error" => error = Some(v.into_owned()),
                        _ => {}
                    }
                }
            }
            let response_body = if let Some(err) = error.as_ref() {
                format!("Authorization failed: {err}. You may close this tab.")
            } else if code.is_some() && state_in.as_deref() == Some(expected_state.as_str()) {
                "Authorization complete. You may close this tab.".to_string()
            } else {
                "Invalid response. You may close this tab.".to_string()
            };
            let response = tiny_http::Response::from_string(response_body).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap(),
            );
            let _ = request.respond(response);

            if let Some(err) = error {
                let _ = tx.send(Err(err));
                return;
            }
            if let (Some(c), Some(s)) = (code, state_in) {
                if s != expected_state {
                    let _ = tx.send(Err("state mismatch in OAuth callback".to_string()));
                    return;
                }
                let _ = tx.send(Ok(c));
                return;
            }
        }
    });

    let code = tokio::task::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(300))
            .map_err(|e| format!("waiting for OAuth callback: {e}"))?
    })
    .await
    .map_err(|e| anyhow!("oauth callback task panicked: {e}"))?
    .map_err(|e| anyhow!("{e}"))?;

    let tok = exchange_code(http, creds, &code, &redirect_uri).await?;
    let expiry = now_unix() + tok.expires_in.unwrap_or(DEFAULT_TOKEN_TTL_SECS);
    Ok(StoredToken {
        access_token: tok.access_token,
        refresh_token: tok.refresh_token,
        expiry,
        scope: tok.scope,
    })
}

pub async fn get_token(
    http: &reqwest::Client,
    token_dir: &Path,
    creds: &Credentials,
    account: &str,
) -> Result<String> {
    if let Some(mut existing) = load_token(token_dir, account)? {
        if existing.expiry > now_unix() + 60 {
            return Ok(existing.access_token);
        }
        if let Some(refresh) = existing.refresh_token.as_deref() {
            let refreshed = refresh_access_token(http, creds, refresh).await?;
            existing.access_token = refreshed.access_token.clone();
            if let Some(rt) = refreshed.refresh_token {
                existing.refresh_token = Some(rt);
            }
            existing.expiry = now_unix() + refreshed.expires_in.unwrap_or(DEFAULT_TOKEN_TTL_SECS);
            if refreshed.scope.is_some() {
                existing.scope = refreshed.scope;
            }
            save_token(token_dir, account, &existing)?;
            return Ok(existing.access_token);
        }
    }

    let fresh = interactive_login(http, creds).await?;
    let access = fresh.access_token.clone();
    save_token(token_dir, account, &fresh)?;
    Ok(access)
}
