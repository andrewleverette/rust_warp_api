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

/// Gets a single customer from the data store
/// 
/// Returns a JSON object of an existing customer. If the customer
/// is not found, it returns a NOT FOUND status code.
/// # Arguments
/// 
/// * `guid` - String -> the id of the customer to retrieve
/// * `db` - `Db` -> the thread safe data store
pub async fn get_customer(guid: String, db: Db) -> Result<Box<dyn warp::Reply>, Infallible> {
    let customers = db.lock().await;

    for customer in customers.iter() {
        if customer.guid == guid {
            return Ok(Box::new(warp::reply::json(customer)))
        }
    }

    Ok(Box::new(StatusCode::NOT_FOUND))
}

/// Updates an existing customer
/// 
/// Overwrites an existing customer in the data store and returns
/// an OK status code. If the customer is not found, a NOT FOUND status
/// code is returned.
/// 
/// # Arguments
/// 
/// * `updated_customer` - `Customer` -> updated customer info
/// * `db` - `Db` -> thread safe data store
pub async fn update_customer(updated_customer: Customer, db: Db) -> Result<impl warp::Reply, Infallible> {
    let mut customers = db.lock().await;

    for customer in customers.iter_mut() {
        if customer.guid == updated_customer.guid {
            *customer = updated_customer;
            return Ok(StatusCode::OK);
        }
    }

    Ok(StatusCode::NOT_FOUND)
}


/// Deletes a customer from the data store
/// 
/// If the customer exists in the data store, the customer is
/// removed and a NO CONTENT status code is returned. If the customer
/// does not exist, a NOT FOUND status code is returned.
/// 
/// # Arguments
/// 
/// * `guid` - String -> the id of the customer to delete
/// * `db` - `Db` -> thread safe data store
pub async fn delete_customer(guid: String, db: Db) -> Result<impl warp::Reply, Infallible> {
    let mut customers = db.lock().await;

    let customer_count = customers.len();

    customers.retain(|customer| {
        customer.guid != guid
    });

    let deleted = customers.len() != customer_count;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}