use std::fs::File;
use std::sync::Arc;

use serde_json::from_reader;
use tokio::sync::Mutex;

use crate::models::Customer;

/// Represents an in memory data store of customer data
pub type Db = Arc<Mutex<Vec<Customer>>>;


/// Initializes the data store
/// 
/// Returns a Db type that either contains customer data
/// or is empty.
pub fn init_db() -> Db {
    let file = File::open("./data/customers.json");
    match file {
        Ok(json) => {
            let customers = from_reader(json).unwrap();
            Arc::new(Mutex::new(customers))
        },
        Err(_) => {
            Arc::new(Mutex::new(Vec::new()))
        }
    }
}