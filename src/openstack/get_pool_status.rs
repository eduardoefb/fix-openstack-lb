use crate::openstack;
use std::error::Error;

pub async fn get_pool_status(token: &str, pool_id: &str) -> Result<String, Box<dyn Error>> {
    let pools = openstack::get_pools(token).await?;
    for pool in pools {
        if pool.id == pool_id {
            return Ok(pool.provisioning_status.clone());
        }
    }
    Ok("none".to_string())
}
