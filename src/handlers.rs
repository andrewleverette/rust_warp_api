use std::convert::Infallible;

use warp::{self, http::StatusCode};

use crate::db::Db;
use crate::models::Customer;

/// Returns a list of customers as JSON
/// 
/// # Arguments
/// 
/// * `db` - `Db` -> thread safe vector of Customer objects
pub async fn list_customers(db: Db) -> Result<impl warp::Reply, Infallible> {
    let customers = db.lock().await;
    let customers: Vec<Customer> = customers.clone();
    Ok(warp::reply::json(&customers))
}

/// Creates a new customer
/// 
/// Adds a new customer object to the data store if the customer
/// doesn't already exist
/// 
/// # Arguments
/// 
/// * `new_customer` - `Customer` type
/// * `db` - `Db` -> thread safe vector of Customer objects
pub async fn create_customer(new_customer: Customer, db: Db) -> Result<impl warp::Reply, Infallible> {
    let mut customers = db.lock().await;

    for customer in customers.iter() {
        if customer.guid == new_customer.guid {
            return Ok(StatusCode::BAD_REQUEST)
        }
    }

    customers.push(new_customer);

    Ok(StatusCode::CREATED)
}

pub async fn show_customer(guid: String, db: Db) -> Result<Box<dyn warp::Reply>, Infallible> {
    let customers = db.lock().await;

    for customer in customers.iter() {
        if customer.guid == guid {
            return Ok(Box::new(warp::reply::json(customer)))
        }
    }

    Ok(Box::new(StatusCode::BAD_REQUEST))
}