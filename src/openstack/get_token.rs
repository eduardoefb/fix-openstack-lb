use reqwest::header::{CONTENT_TYPE};
use std::env;
use std::error::Error;

/// Retrieves the token from the OpenStack Keystone API using environment variables.
pub async fn get_token() -> Result<String, Box<dyn Error>> {
    // Retrieve environment variables
    let os_auth_url = env::var("OS_AUTH_URL")?;
    let os_user_domain_name = env::var("OS_USER_DOMAIN_NAME")?;
    let os_username = env::var("OS_USERNAME")?;
    let os_password = env::var("OS_PASSWORD")?;
    let os_project_name = env::var("OS_PROJECT_NAME")?;
    
    // Construct the full Keystone URL
    let url = format!("{}/auth/tokens", os_auth_url);

    // JSON payload
    let payload = serde_json::json!({
        "auth": {
            "identity": {
                "methods": ["password"],
                "password": {
                    "user": {
                        "name": os_username,
                        "domain": { "id": os_user_domain_name },
                        "password": os_password
                    }
                }
            },
            "scope": {
                "project": {
                  "name": os_project_name,
                  "domain": { "id": os_user_domain_name }
                }
              }            
        }
    });

    // Create an HTTP client
    let client = reqwest::Client::new();

    // Send the POST request
    let response = client
        .post(&url)
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await?;

    // Check the response and extract the token
    if response.status().is_success() {
        if let Some(token) = response.headers().get("X-Subject-Token") {
            return Ok(token.to_str()?.to_string());
        } else {
            return Err("X-Subject-Token not found in response headers".into());
        }
    } else {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }
}
