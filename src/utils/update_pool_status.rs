use mysql::{Opts, Pool, params};
use mysql::prelude::Queryable;
use std::error::Error;
use ini::Ini;
use crate::utils;

pub fn update_pool_status(pool_id: &str) -> Result<(), Box<dyn Error>> {
    let conf = Ini::load_from_file("/etc/octavia/octavia.conf").unwrap();
    let db_section = conf.section(Some("database")).unwrap();
    let conn_str = db_section.get("connection").unwrap().replace("mysql+pymysql", "mysql");

    // Parse the connection string into `Opts`
    let opts = Opts::from_url(&conn_str)
        .map_err(|e| format!("Invalid connection string: {}", e))?;

    // Create a connection pool
    let pool = Pool::new(opts)
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    // Get a connection from the pool
    let mut conn = pool.get_conn()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    // Execute the SQL query
    conn.exec_drop(
        r"UPDATE pool SET provisioning_status='ACTIVE' WHERE provisioning_status='PENDING_CREATE' AND id=:pool_id",
        params! {
            "pool_id" => pool_id,
        },
    )
    .map_err(|e| format!("Failed to update pool status: {}", e))?;

    println!("{} Successfully updated pool status for pool ID: {}", utils::get_timestamp(), pool_id);

    Ok(())

}