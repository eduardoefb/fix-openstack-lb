use crate::utils;
const ERROR_ATTEMPTS: i32 = 30;
const DELAY_BETWEEN_ERRORS: u64 = 1;

use reqwest::header::{HeaderMap, HeaderValue};
use std::error::Error;
use tokio::time::{sleep, Duration};

pub async fn recreate_member(
    token: &str,
    pool_id: &str,
    member_id: &str,
    member_data: &serde_json::Value,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Construct the base URL using the same logic as `get_members`
    let os_auth_url = std::env::var("OS_AUTH_URL")?;
    let base_url = format!(
        "{}://{}",
        url::Url::parse(&os_auth_url)?.scheme(),
        url::Url::parse(&os_auth_url)?.host_str().ok_or("Invalid OS_AUTH_URL format")?
    );

    // DELETE the member with retry logic
    let delete_url = format!("{base_url}:9876/v2.0/lbaas/pools/{pool_id}/members/{member_id}");
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("X-Auth-Token", HeaderValue::from_str(token)?);

    let mut delete_attempts = 0;
    while delete_attempts < ERROR_ATTEMPTS {
        let delete_response = client
            .delete(&delete_url)
            .headers(headers.clone())
            .send()
            .await?;

        if delete_response.status().is_success() {
            println!("{} Successfully deleted member: {}", utils::get_timestamp(), member_id);
            break;
        } else if delete_response.status() == reqwest::StatusCode::CONFLICT {
            delete_attempts += 1;
            println!(
                "{} Delete attempt {} failed with status 409 Conflict. Retrying... ({} of {})",
                utils::get_timestamp(),
                delete_attempts,
                delete_attempts,
                ERROR_ATTEMPTS
            );
            sleep(Duration::from_secs(DELAY_BETWEEN_ERRORS)).await;
        } else {
            return Err(format!(
                "Failed to delete member: {} - {}",
                member_id,
                delete_response.status()
            )
            .into());
        }

        if delete_attempts == 30 {
            return Err(format!(
                "Failed to delete member after {} attempts: {}",ERROR_ATTEMPTS,
                member_id
            )
            .into());
        }
    }

    // CREATE the member with retry logic
    let create_url = format!("{base_url}:9876/v2.0/lbaas/pools/{pool_id}/members");
    let mut create_attempts = 0;
    while create_attempts < ERROR_ATTEMPTS {
        let create_response = client
            .post(&create_url)
            .headers(headers.clone())
            .json(member_data)
            .send()
            .await?;

        if create_response.status().is_success() {
            println!("{} Successfully recreated member: {}", utils::get_timestamp(), member_id);
            return Ok(());
        } else {
            create_attempts += 1;
            println!(
                "{} Create attempt {} failed with status {}. Retrying... ({} of {})",
                utils::get_timestamp(),
                create_attempts,
                create_response.status(),
                create_attempts,
                ERROR_ATTEMPTS
            );
            sleep(Duration::from_secs(DELAY_BETWEEN_ERRORS)).await;
        }

        if create_attempts == 30 {
            return Err(format!(
                "Failed to create member after {} attempts: {}",ERROR_ATTEMPTS,
                member_id
            )
            .into());
        }
    }

    Ok(())
}
