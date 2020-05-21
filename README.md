# Rust Warp API Example

## Overview

This is an example project that uses Warp and Tokio to build a simple asynchronous api.

## Goals

1. Become familiar with the Warp framework.
2. Become more familiar with using async/await in Rust 
3. Get more comfortable with Rust's Trait system
4. Get a better understanding of API design in Rust

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
get_customer -> return the details of a single customer
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

Also, the database module will need to have the ability to initialize the data store once the server starts.

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
- get_customer -> return the details of a single customer
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

##### Get Customer

The `get_customer` handler will take a guid and a data store reference as a parameter returns a JSON object of the customer if it is found else it returns a default customer.

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
pub async fn get_customer(guid: String, db: Db) -> Result<Box<dyn warp::Reply>, Infallible> {
    
}
```

The return type is a little different than the other functions. The reason is that we need to be able to return either a JSON object or a status code that indicates a not found error. Since `warp::reply::json()` and `StatusCode` implement the `warp::Reply` trait, we can use [dynamic dispatching](https://doc.rust-lang.org/1.8.0/book/trait-objects.html) to return the appropriate type.

With the proper return type, our function body is fairly straightforward:

```rust
pub async fn get_customer(guid: String, db: Db) -> Result<Box<dyn warp::Reply>, Infallible> {
    let customers = db.lock().await;

    for customer in customers.iter() {
        if customer.guid == guid {
            return Ok(Box::new(warp::reply::json(customer)))
        }
    }

    Ok(Box::new(StatusCode::NOT_FOUND))
}
```

##### Update Customer

The `update_customer` handler will take a customer and a data store reference as an argument and returns a status code of OK if the customer is found and updated or NOT FOUND if the customer is not in the data store.

The function should look like this:

```rust
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
```

##### Delete Customer

The `delete_customer` handler will take a guid and a reference to the data store as an argument. The function will remove the customer with a matching guid and return a NO CONTENT status code. If a match is not found then it will return a NOT FOUND status code.

The function should look something like this:

```rust
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
```

#### Routes

We now have all the handler functions implemented. Next we need to piece together the routes that will call the handlers.

In `main.rs`, define another module:

```rust
mod routes;
```

Then we create a file called `routes.rs` in the `src` directory and add the following:

```rust
use std::convert::Infallible;
use warp::{self, Filter};

use crate::db::Db;
use crate::handlers;
use crate::models::Customer;
```

First we need a helper function to pass a reference of the data store into the handlers from the routes.

Add the following to `routes.rs`:

```rust
fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = Infallible> {
    warp::any().map(move || db.clone())
}
```

This function allows the data store be injected into the route and passed along into the handler. `Filter` is a trait in the warp library. The `Filter` trait provides functionality to compose routes that are the result of one or more `Filter` methods. This will make more sense with an example.

Just for a reminder, here are the routes we need to define:

```
/customers
    - GET -> list all customers in data store
    - POST -> create new customer and insert into data store
/customers/{guid}
    - GET -> list info for a customer
    - POST -> update information for a customer
    - DELETE -> remove customer from data store
```

##### GET /customers

The first route will simply get all customers in the data store. Add the following to the `routes.rs`:

```rust
pub fn customers_list(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("customers")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::list_customers)
}
```

The function returns a type that implements the `Filter` trait. The `Extract` is used when a match occurs and the value of the `Extract` is returned.

Basically the function is defining a route that matches when the requested path is "/customers" and it is a GET request. 

Also, to save some work for later, I'll implement another function that will serve as a wrapper for all the customer routes. It will make it easier later when we hook everything together.

So add the following to `routes.rs`:

```rust
pub fn customer_routes(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    customers_list(db.clone())
}
```

##### POST /customers

This route will add a new customer to the data store if it doesn't already exist.

One thing to add before we add the function for the route is a helper function to extract the JSON from the POST request body.

Add the following to `routes.rs`:

```rust
fn json_body() -> impl Filter<Extract = (Customer,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}
```

The function will be very similar to `customers_list` except for the handler. Add the following to `routes.rs`:

```rust
pub fn create_customer(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("customers")
        .and(warp::post())
        .and(json_body())
        .and(with_db(db))        
        .and_then(handlers::create_customer)
}
```

This function defines a route the matches when the path is "/customers" and it is a post request. Then the JSON from the post request and the data store reference is extracted and passed in to the handler.

##### GET /customers/{guid}

This route will attempt to retrieve a single customer from the data store.

This route function will introduce the `path!` macro from `warp`. This macro enables us to create a path with a variable.

Add the following to `routes.rs`:

```rust
pub fn get_customer(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("customers" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_customer)
}
```

This defines a route the will match on "customers/{some string value} and a GET request. It then extracts the data store and passes it into the handler.

One thing to consider for routes is that the most specific route should be checked first otherwise a route may not be matched.

For example if the helper function for the routes is updated to this:

```rust
pub fn customer_routes(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    customers_list(db.clone())
        .or(create_customer(db.clone()))
        .or(get_customer(db.clone()))
}
```

The `get_customer` route will never match because the share a common root path - "/customers" - which means the customer list route will match "/customers" and "/customers/{guid}".

To fix the mismatch issue, arrange the route so the most specific match is first. Like this:

```rust
pub fn customer_routes(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_customer(db.clone())
        .or(customers_list(db.clone()))
        .or(create_customer(db.clone()))
}
```

##### PUT /customers/{guid}

This route will attempt to update a customer if it exists and return an OK status code, otherwise a NOT FOUND status code is returned.

The route will look similar to the create customer route but it will match a different path. Add the following to `routes.rs`:

```rust
pub fn update_customer(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("customers" / String)
        .and(warp::put())
        .and(json_body())
        .and(with_db(db))
        .and_then(handlers::update_customer)
}
```

Then update the customer route wrapper:

```rust
pub fn customer_routes(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_customer(db.clone())
        .or(update_customer(db.clone()))
        .or(create_customer(db.clone()))
        .or(customers_list(db))
}
```

##### DELETE /customers/{guid}

The last route simply deletes a customer from the data store if it matches the given guid and then returns a NO CONTENT status code, otherwise a NOT FOUND status code is returned.

Add the following to `routes.rs`:

```rust
fn delete_customer(
    db: Db
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("customers" / String)
        .and(warp::delete())
        .and(with_db(db))
        .and_then(handlers::delete_customer)
}
```

And then update the customer route wrapper:

```rust
pub fn customer_routes(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_customer(db.clone())
        .or(update_customer(db.clone()))
        .or(delete_customer(db.clone()))
        .or(create_customer(db.clone()))
        .or(customers_list(db))
}
```

This finishes up all the routes. Now we can move on to tying everything together.

#### Main

The `main.rs` will pull all of the pieces together. It will initialize the data store, get all the routes, and start the server. It's also fairly short file, so I'll just show the whole thing:

```rust
use warp;

mod db;
mod handlers;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    let db = db::init_db();
    let customer_routes = routes::customer_routes(db);

    warp::serve(customer_routes)
        .run(([127, 0, 0, 1], 3000))
        .await;
}
```

We've already seen the first few lines, so lets go through the main function. 

The function attribute `#[tokio::main]` sets the entry point for the `tokio` runtime. This allows us to declare the `main` function as `async`.

The first two lines of `main` are just calling functions from our modules. The first initializes the data store and the second gets our customer routes wrapper.

The last line uses `warp::server` to create a server and then `run` to start the server on the provided host and port. We use the `await` keyword to yield until the `run` function is finished.

### Review

This completes a simple API using Rust and the Warp framework. There are improvements that can be made however. 

Here are a couple of ideas:

- Testing can be added to confirm that the endpoints are behaving as expected
- Functionality can added to the `db` module to allow for saving the data store by overwriting the JSON file.
- The simple data store could be replaced with an actual database like PostgreSQL or even MongoDB.