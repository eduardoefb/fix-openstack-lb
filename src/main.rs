mod openstack;
mod utils;
//const CHECK_TIMES: i32 = 12;
//const CHECK_DELAY: u64 = 10;

const CHECK_TIMES: i32 = 6;
const CHECK_DELAY: u64 = 5;

use serde_json::json;
use std::error::Error;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
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

    // Check if there is any loadbalancer in "PENDING_CREATE"
    loop {
        let lb_list = openstack::get_loadbalancers(&token).await?;
        let mut pending_lb = false;
        for lb in lb_list{
            println!("{} LB: {} ,LB Status:{}", utils::get_timestamp(), lb.name, lb.provisioning_status );
            if lb.provisioning_status != "ACTIVE"{
                pending_lb = true;
            }
        }
        if ! pending_lb {
            break
        }
        sleep(Duration::from_secs(CHECK_DELAY)).await;
    }

    // Execute a first check and change pool to pending_create if there is any provisioning_status not active for the members
    let pools = openstack::get_pools(&token).await?;
    for pool in pools {    
        println!("{} Pool ID = {} has {} state.", utils::get_timestamp(), pool.id, pool.provisioning_status);
        match openstack::get_members(&token, &pool.id).await {
            Ok(members) => {
                for member in members {
                    //println!("Member: {:?}", member);
                    println!(
                        "{} Checking Member: {} - {} [{}:{}]",
                        utils::get_timestamp(),
                        member.id,
                        member.name,
                        member.address,
                        member.protocol_port
                    );
                    if member.provisioning_status != "ACTIVE" {
                        match utils::update_pool_status_to_pending(&pool.id) {
                            Ok(_) => println!("{} Successfully updated pool {} to PENDING_CREATE", utils::get_timestamp(), pool.id),
                            Err(err) => eprintln!("{} Error updating pool {}: {}", utils::get_timestamp(), pool.id, err),
                        }
                    }
                }
            }
            Err(err) => eprintln!(
                "{} Error retrieving members for pool {}: {}",
                utils::get_timestamp(),
                pool.id,
                err
            ),            
        }
        sleep(Duration::from_secs(CHECK_DELAY)).await;
    }

    // Start the verification and recreation if needed
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

                match utils::update_pool_status_to_active(&pool.id) {
                    Ok(_) => println!("{} Successfully updated pool {} to ACTIVE", utils::get_timestamp(), pool.id),
                    Err(err) => eprintln!("{} Error updating pool {}: {}", utils::get_timestamp(), pool.id, err),
                }

                let token_clone = token.clone();
                let pool_id = pool.id.clone();
                tokio::spawn(async move {
                    match openstack::get_members(&token_clone, &pool_id).await {
                        Ok(members) => {
                            for member in members {
                                println!(
                                    "{} Recreating Member: {} - {} [{}:{}]",
                                    utils::get_timestamp(),
                                    member.id,
                                    member.name,
                                    member.address,
                                    member.protocol_port
                                );
                
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
                                
                                if member.provisioning_status != "ACTIVE"{
                                    match openstack::recreate_member(
                                        &token_clone,
                                        &pool_id,
                                        &member.id,
                                        &new_member_data,
                                    ).await{
                                        Ok(_) => println!(
                                            "{} Successfully recreated member: {}",
                                            utils::get_timestamp(),
                                            member.id
                                        ),
                                        Err(err) => eprintln!(
                                            "{} Error recreating member {}: {}",
                                            utils::get_timestamp(),
                                            member.id,
                                            err
                                        ),
                                    }
                                }
                            }
                        }
                        Err(err) => eprintln!(
                            "{} Error retrieving members for pool {}: {}",
                            utils::get_timestamp(),
                            pool_id,
                            err
                        ),
                    }
                });
            }
        }

    }

    // Wait until the update is finished
    
    loop{
        let mut no_pending = true;
        sleep(Duration::from_secs(CHECK_DELAY)).await; 
        let pools = openstack::get_pools(&token).await?;
        for pool in pools {
            println!("{} Pool ID = {} has {} state.", utils::get_timestamp(), pool.id, pool.provisioning_status);
            match openstack::get_members(&token, &pool.id).await {
                Ok(members) => {
                    for member in members {
                        //println!("Member: {:?}", member);
                        println!(
                            "{} Checking Member: {} - {} [{}:{}]",
                            utils::get_timestamp(),
                            member.id,
                            member.name,
                            member.address,
                            member.protocol_port
                        );

                        if member.provisioning_status != "ACTIVE" && pool.provisioning_status != "PENDING_CREATE" {
                            no_pending = false
                        }
                        println!("{} Member id: {}, status: {}, nopending: {}", utils::get_timestamp(), member.id, member.provisioning_status, no_pending);
                        
                
                    }
                }
                Err(err) => eprintln!(
                    "{} Error retrieving members for pool {}: {}",
                    utils::get_timestamp(),
                    pool.id,
                    err
                ),            
            }
        }

        if no_pending {
            break
        }
    }

    Ok(())
}
