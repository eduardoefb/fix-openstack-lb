mod openstack;
mod utils;
const CHECK_TIMES: i32 = 5;
const CHECK_DELAY: u64 = 3;

use serde_json::json;
use std::error::Error;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token: String;

    // Get the token
    match openstack::get_token().await {
        Ok(t) => {
            token = t;
        }
        Err(err) => {
            return Err(err);
        }
    }
        
    let pools = openstack::get_pools(&token).await?;

    for pool in pools {
        println!("{} Pool ID = {} has {} state.", utils::get_timestamp(), pool.id, pool.provisioning_status);

        if pool.provisioning_status == "PENDING_CREATE"{
            let mut check_times = 0;
            while check_times < CHECK_TIMES {
                let p_status = openstack::get_pool_status(&token, &pool.id).await?;
                if p_status == "ACTIVE"{
                    break;
                }
                println!("{} Check ({} of {}): Pool {} is still in {}", utils::get_timestamp(), check_times, CHECK_TIMES, pool.id, p_status);
                sleep(Duration::from_secs(CHECK_DELAY)).await;
                check_times += 1;
            }
            if check_times >= CHECK_TIMES{

                match utils::update_pool_status(&pool.id) {
                    Ok(_) => println!("{} Successfully updated pool {} to ACTIVE", utils::get_timestamp(), pool.id),
                    Err(err) => eprintln!("{} Error updating pool {}: {}", utils::get_timestamp(), pool.id, err),
                }

                match openstack::get_members(&token, &pool.id).await {
                    Ok(members) => {
                        for member in members {
                            println!(
                                "{} Recreating Member: {} - {} [{}:{}]",
                                utils::get_timestamp(), member.id, member.name, member.address, member.protocol_port
                            );

                            // Define new member data for recreation
                            let new_member_data = json!({"member": {
                                "name": member.name,
                                "weight": member.weight,
                                "admin_state_up": member.admin_state_up,
                                "subnet_id": member.subnet_id,
                                "address": member.address,
                                "protocol_port": member.protocol_port,
                                "monitor_port": member.monitor_port,
                                "backup": member.backup,
                                "tags": member.tags
                            }});

                            // Recreate the member
                            match openstack::recreate_member(
                                &token,
                                &pool.id,
                                &member.id,
                                &new_member_data,
                            )
                            .await
                            {
                                Ok(_) => println!("{} Successfully recreated member: {}", utils::get_timestamp(), member.id),
                                Err(err) => eprintln!(
                                    "{} Error recreating member {}: {}",
                                    utils::get_timestamp(), member.id, err
                                ),
                            }
                        }
                    }
                    Err(err) => eprintln!("{} Error retrieving members for pool {}: {}", utils::get_timestamp(), pool.id, err),
                }                

            }            
        }

    }
    Ok(())
}
