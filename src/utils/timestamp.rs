use chrono::Local;

pub fn get_timestamp() -> String{    
    Local::now().format("%b %d %H:%M:%S").to_string().to_string()
}