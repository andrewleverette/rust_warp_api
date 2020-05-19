use std::convert::Infallible;

use warp;

use crate::db::Db;
use crate::models::Customer;

// Returns a list of customers as JSON
pub async fn list_customers(db: Db) -> Result<impl warp::Reply, Infallible> {
    let customers = db.lock().await;
    let customers: Vec<Customer> = customers.clone();
    Ok(warp::reply::json(&customers))
}
