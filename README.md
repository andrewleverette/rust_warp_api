# Rust Warp API Example

## Overview

This is an example project that uses Warp and Tokio to build a simple asynchronous api.

## Goals

1. Become familiar with the Warp framework.
2. Become more familiar with using async/await in Rust 
3. Get a better understanding of API design in Rust

## Notes

### Design

#### Routes

```
/customers
    - GET -> list all customers in data store
    - POST -> create new customer and insert into data store
/customers/{guid}
    - GET -> list info for a customer
    - POST -> update information for a customer
    - DELETE -> remove customer from data store
```

#### Handlers

Based on the defined routes, I will need the following handlers:

```
list_customers -> return a list all customers in database
create_customer -> create a new customer and add it to the database
show_customer -> return the details of a single customer
update_customer -> update the details of a single customer
delete_customer -> delete a customer from the database
```

#### Database

For right now, I'll just use an in-memory data store to share across the route handlers.

I used [Mockaroo](https://www.mockaroo.com/) to generate a JSON data set of customer data. The data is a JSON array where each object has the following structure:

```json
{
    "guid": "String",
    "first_name": "String",
    "last_name": "String",
    "email": "String",
    "address": "String"
}
```

Also, the database module will need to have the ability to initialize and save it's current state.

### Dependencies

As of right now, I know that I will need the following dependencies:

* Warp - A web server framework for Rust
* Tokio - An asynchronous run-time for Rust
* Serde - A de/serialization library for converting JSON to typed data and vice versa.

### Implementation


#### Models

The first thing I want to do is define my customer model and also start adding some structure to the code.

In `main.rs`, define a new module called `models` like this:

```rust
mod models;

fn main() {
    // ...
}
```

Then create a new file called `models.rs` and add the following:

```rust
pub struct Customer {
    pub guid: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub address: String,
}
```

Since I'm designing an API, this data structure needs be able to covert to and from JSON. I also want to be able to copy the structure into and out of the data store without having to worry about the borrow checker. 

To accomplish this I'll add a derive statement to use a couple of the macros from the Serde library and a couple from Rust.

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Customer {
    pub guid: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub address: String,
}
```

#### Database

The database for this example API will be an in-memory database that is a vector of the the `Customer` model. However, the data store will need to be shared across multiple routes, so we can use Rust's [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) smart pointer along with a [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) to allow for thread safety.

First, update `main.rs` with a new module called `db`:

```rust
mod db;
mod models;

fn main() {
    // ...
}
```

Then create a new file called `db.rs`.

There are a few things to do in this file, but the first thing to do is to define what the data store will look like.

A simple data store is just a vector of `Customer` structs, but it needs to be wrapped in a thread safe reference to be able to use multiple references of the data store in multiple asynchronous handlers.

Add the following to `db.rs`:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::Customer;

pub type Db = Arc<Mutex<Vec<Customer>>>;
```

Now that we have defined the structure of the data store, we need a way to initialize the data store. Initializing the data store has two outcomes, either an empty data store or a data store loaded with data from a data file.

An empty store is rather straight forward.

```rust
pub fn init_db() -> Db {
    Arc::new(Mutex::new(Vec::new()))
}
```

But in order to load data from a file, we need to add another dependency.

Add the following to the `Cargo.toml` file:

```toml
serde_json = "1.0"
```

Now we can update `db.rs` with the following:

```rust
use std::fs::File;
use serde_json::from_reader;

pub fn init_db() -> Db {
    let file = File::open("./data/customers.json");
    match file => {
        Ok(json) => {
            let customers = from_reader(json).unwrap();
            Arc::new(Mutex::new(customers))
        },
        Err(_) => {
            Arc::new(Mutex::new(Vec::new()))
        }
    }
}
```

This function attempts to read from the file at `./data/customers.json`. If it is successful, the function returns a data store loaded with the customer data, else it returns an empty vector.

The `db.rs` should look like this now:

```rust
use std::fs::File;
use std::sync::Arc;

use serde_json::from_reader;
use tokio::sync::Mutex;

use crate::models::Customer;

pub type Db = Arc<Mutex<Vec<Customer>>>;

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
```

#### Handlers

At this point we have the models and the database setup. Now we need a way to tie them together. That's were the handlers come in.

First lets define a new module in `main.rs` and create a new file called `handlers.rs`.

```rust
mod handlers;
```

We also need to add a couple of imports. In the `handlers.rs` file add the following:

```rust
use std::convert::Infallible;
use warp;

use crate::models::Customer;
use crate::db::Db;
```

This snippet makes the `Customer` model and `Db` type we have defined in the other modules available in the `handlers` module. It also imports the root `warp` module and the [`Infallible`](https://doc.rust-lang.org/std/convert/enum.Infallible.html) enum, which is the error type for errors that can never happen.

Now as a reminder, here are the handlers we want to implement:

- list_customers -> return a list all customers in database
- create_customer -> create a new customer and add it to the - database
- show_customer -> return the details of a single customer
- update_customer -> update the details of a single customer
- delete_customer -> delete a customer from the database

##### List Customers

The `list_customers` handler will take a reference to the data store as an argument and return a `Result` type that wraps a JSON response.

The function definition will look like this:

```rust
pub async fn list_customers(db: Db) -> Result<impl warp::Reply, Infallible> {
   // ... 
}
```

For the function body, we need to get the customer list out of the data store and return it as a JSON object. For convenience, `warp` provides a reply method that will convert a vector to a json object.

Update the function with the following:

```rust
pub async fn list_customers(db: Db) -> Result<impl warp::Reply, Infallible> {
    let customers = db.lock().await;
    let customers: Vec<Customer> = customers.clone();
    Ok(warp::reply::json(&customers))
}
```

The line `let customers = db.lock().await;` causes the the current task to yield until a lock can be acquired and the data store can be referenced safely.

The line `let customers: Vec<Customer> = customers.clone()` takes the inner vector out of the `MutexGaurd`.

The last line `Ok(warp::reply::json(&customers))` wraps a JSON reply in a `Ok` variant of the `Result` type.

##### Create Customer

The `create_customer` handler will take a `Customer` object and a reference to the data store as an argument and return a created status code if the new customer is added to the customer list or a bad request code if the customer already exists.

Before we get to the function, we need to update the warp import statement to allow the use of status codes.

In `handlers.rs`, change the line `use warp;` to the following:

```rust
use warp::{self, http::StatusCode};
```

This will allow the use of `StatusCode` enum as a response.

The function definition will be similar to the `list_customers` handler, so we can just jump into the full definition.

```rust
pub async fn create_customer(new_customer: Customer, db: Db) -> Result<impl warp::Reply, Infallible> {
    let mut customers = db.lock().await;

    for customer in customers.iter() {
        if customer.guid == new_customer.guid {
            return Ok(StatusCode::BAD_REQUEST)
        }
    }

    customers.push(new_customers);

    Ok(StatusCode::Created)
}
```

##### Show Customer

The `show_customer` handler will take a guid and a data store reference as a parameter returns a JSON object of the customer if it is found else it returns a default customer.

Before we write this implementation, we need to add one macro to the `Customer` struct. Update the `Customer` struct in `models.rs` to the following:

```rust
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Customer {
    pub guid: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub address: String,
}
```

The function definition looks like this:

```rust
pub async fn show_customer(guid: String, db: Db) -> Result<Box<dyn warp::Reply>, Infallible> {
    
}
```

The return type is a little different than the other functions. The reason is that we need to be able to return either a JSON object or a status code that indicates a bad request. Since `warp::reply::json()` and `StatusCode` implement the `warp::Reply` trait, we can use dynamic dispatching to return the appropriate type.

With the proper return type, our function body is fairly straightforward:

```rust
pub async fn show_customer(guid: String, db: Db) -> Result<Box<dyn warp::Reply>, Infallible> {
    let customers = db.lock().await;

    for customer in customers.iter() {
        if customer.guid == guid {
            return Ok(Box::new(warp::reply::json(customer)))
        }
    }

    Ok(Box::new(StatusCode::BAD_REQUEST))
}
```