#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub sub: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub picture: Option<String>,
    #[serde(default)]
    pub locale: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgcPost {
    pub id: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub lifecycle_state: Option<String>,
    #[serde(default)]
    pub created: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgcPostsResponse {
    #[serde(default)]
    pub elements: Vec<UgcPost>,
    #[serde(default)]
    pub paging: Option<Paging>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    #[serde(default)]
    pub count: Option<i64>,
    #[serde(default)]
    pub start: Option<i64>,
    #[serde(default)]
    pub total: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    #[serde(default)]
    pub name: Option<Value>,
    #[serde(default)]
    pub description: Option<Value>,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub staff_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionsResponse {
    #[serde(default)]
    pub elements: Vec<Connection>,
    #[serde(default)]
    pub paging: Option<Paging>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPosting {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub company_name: Option<String>,
    #[serde(default)]
    pub apply_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPostingsResponse {
    #[serde(default)]
    pub elements: Vec<JobPosting>,
    #[serde(default)]
    pub paging: Option<Paging>,
}
