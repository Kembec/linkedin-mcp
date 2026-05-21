use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Method;
use serde_json::{json, Value};
use urlencoding::encode;

pub const API_BASE: &str = "https://api.linkedin.com";

pub struct LinkedInClient {
    http: reqwest::Client,
    access_token: String,
}

impl LinkedInClient {
    pub fn new(http: reqwest::Client, access_token: String) -> Self {
        Self { http, access_token }
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.access_token))
                .context("invalid access token for header")?,
        );
        headers.insert(
            "X-Restli-Protocol-Version",
            HeaderValue::from_static("2.0.0"),
        );
        headers.insert("LinkedIn-Version", HeaderValue::from_static("202312"));
        Ok(headers)
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&Value>,
    ) -> Result<Value> {
        let url = format!("{API_BASE}{path}");
        let mut req = self
            .http
            .request(method.clone(), &url)
            .headers(self.headers()?);
        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(b) = body {
            req = req.json(b);
        }
        let resp = req
            .send()
            .await
            .with_context(|| format!("HTTP {method} {url}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!("LinkedIn API {status} on {method} {url}: {text}"));
        }
        if text.is_empty() {
            return Ok(json!({ "ok": true }));
        }
        serde_json::from_str(&text).with_context(|| format!("parse JSON from {url}"))
    }

    pub async fn get_userinfo(&self) -> Result<Value> {
        self.request(Method::GET, "/v2/userinfo", &[], None).await
    }

    pub async fn create_ugc_post(
        &self,
        author_urn: &str,
        text: &str,
        visibility: &str,
    ) -> Result<Value> {
        let member_network_visibility = match visibility {
            "CONNECTIONS" => "CONNECTIONS",
            "LOGGED_IN" => "LOGGED_IN",
            _ => "PUBLIC",
        };
        let body = json!({
            "author": author_urn,
            "lifecycleState": "PUBLISHED",
            "specificContent": {
                "com.linkedin.ugc.ShareContent": {
                    "shareCommentary": {
                        "text": text
                    },
                    "shareMediaCategory": "NONE"
                }
            },
            "visibility": {
                "com.linkedin.ugc.MemberNetworkVisibility": member_network_visibility
            }
        });
        self.request(Method::POST, "/v2/ugcPosts", &[], Some(&body))
            .await
    }

    pub async fn get_own_posts(&self, author_urn: &str, count: i64, start: i64) -> Result<Value> {
        let authors_param = format!("List({author_urn})");
        let query = [
            ("q", "authors".to_string()),
            ("authors", authors_param),
            ("count", count.to_string()),
            ("start", start.to_string()),
        ];
        self.request(Method::GET, "/v2/ugcPosts", &query, None)
            .await
    }

    pub async fn delete_post(&self, post_id: &str) -> Result<Value> {
        let encoded = encode(post_id);
        let path = format!("/v2/ugcPosts/{encoded}");
        self.request(Method::DELETE, &path, &[], None).await
    }

    pub async fn get_organization(&self, org_id: &str) -> Result<Value> {
        let path = format!("/v2/organizations/{org_id}");
        self.request(Method::GET, &path, &[], None).await
    }

    pub async fn get_connections(&self, start: i64, count: i64) -> Result<Value> {
        let query = [
            ("q", "viewer".to_string()),
            ("start", start.to_string()),
            ("count", count.to_string()),
        ];
        self.request(Method::GET, "/v2/connections", &query, None)
            .await
    }

    pub async fn search_jobs(&self, keywords: &str, count: i64) -> Result<Value> {
        let query = [
            ("keywords", keywords.to_string()),
            ("count", count.to_string()),
        ];
        self.request(Method::GET, "/v2/jobPostings", &query, None)
            .await
    }

    pub async fn create_article(
        &self,
        author_urn: &str,
        title: &str,
        content: &str,
        commentary: &str,
        visibility: &str,
        draft: bool,
    ) -> Result<Value> {
        let member_network_visibility = match visibility {
            "CONNECTIONS" => "CONNECTIONS",
            "LOGGED_IN" => "LOGGED_IN",
            _ => "PUBLIC",
        };
        let lifecycle_state = if draft { "DRAFT" } else { "PUBLISHED" };
        let body = json!({
            "author": author_urn,
            "lifecycleState": lifecycle_state,
            "title": title,
            "content": content,
            "commentary": commentary,
            "visibility": {
                "com.linkedin.ugc.MemberNetworkVisibility": member_network_visibility
            }
        });
        self.request(Method::POST, "/v2/articles", &[], Some(&body))
            .await
    }

    pub async fn send_message(&self, recipient_urn: &str, message_body: &str) -> Result<Value> {
        let body = json!({
            "recipients": [recipient_urn],
            "body": message_body
        });
        self.request(Method::POST, "/v2/conversations", &[], Some(&body))
            .await
    }
}
