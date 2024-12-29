use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::env;
use std::error::Error;
use url::Url;

// Define the structure to hold the pool details
#[derive(Debug, Deserialize)]
pub struct Pool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provisioning_status: String,
    pub operating_status: String,
    pub protocol: String,
    pub lb_algorithm: String,
    pub project_id: String,
    pub members: Vec<Member>,
}

#[derive(Debug, Deserialize)]
pub struct Member {
    pub id: String,
}

// Define the top-level response structure
#[derive(Debug, Deserialize)]
struct PoolsResponse {
    pools: Vec<Pool>,
}


pub async fn get_pools(token: &str) -> Result<Vec<Pool>, Box<dyn Error>> {
    // Get the OS_AUTH_URL environment variable
    let os_auth_url = env::var("OS_AUTH_URL")?;

    // Parse the URL to extract the scheme and host
    let parsed_url = Url::parse(&os_auth_url)?;
    let base_url = format!(
        "{}://{}",
        parsed_url.scheme(),
        parsed_url.host_str().ok_or("Invalid URL: Missing host")?
    );

    // Construct the lbaas URL
    let url = format!("{base_url}:9876/v2.0/lbaas/pools");

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
        let body = response.json::<PoolsResponse>().await?;
        Ok(body.pools)
    } else {
        Err(format!("Failed to retrieve pools: {}", response.status()).into())
    }
}

