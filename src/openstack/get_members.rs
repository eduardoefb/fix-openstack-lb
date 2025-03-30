use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::error::Error;

// Define the structure to hold the member details
#[derive(Debug, Deserialize)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub operating_status: String,
    pub provisioning_status: String,
    pub admin_state_up: bool,
    pub address: String,
    pub protocol_port: u16,
    pub weight: u8,
    pub backup: bool,
    pub subnet_id: String,
    pub project_id: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub monitor_address: Option<String>,
    pub monitor_port: Option<u16>,
    pub tags: Vec<String>,
}

// Define the top-level response structure
#[derive(Debug, Deserialize)]
struct MembersResponse {
    members: Vec<Member>,
}

pub async fn get_members(token: &str, pool_id: &str) -> Result<Vec<Member>, Box<dyn Error + Send + Sync>> {
    // Construct the URL for the members endpoint
    let os_auth_url = std::env::var("OS_AUTH_URL")?;
    let base_url = format!(
        "{}://{}",
        url::Url::parse(&os_auth_url)?.scheme(),
        url::Url::parse(&os_auth_url)?.host_str().ok_or("Invalid OS_AUTH_URL format")?
    );
    let url = format!("{base_url}:9876/v2.0/lbaas/pools/{pool_id}/members");

    // Create a client
    let client = reqwest::Client::new();

    // Build headers with the token
    let mut headers = HeaderMap::new();
    headers.insert("X-Auth-Token", HeaderValue::from_str(token)?);

    // Send the GET request
    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await?;

    // Check for success and deserialize JSON
    if response.status().is_success() {
        let body = response.json::<MembersResponse>().await?;        
        Ok(body.members)
    } else {
        Err(format!("Failed to retrieve members: {}", response.status()).into())
    }
}
