# Rust Warp API Example

## Overview

This is an example project that uses Warp and Tokio to build a simple asynchronous api.

## Goals

1. Become familiar with the Warp framework.
2. Get a better understanding of API design in Rust

## Notes

### Design

#### Endpoints

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

Based on the defined endpoints, I will need the following handlers:

> These are just function stubs
```rust
pub async fn list_customers() -> {}
pub async fn create_customer() -> {}
pub async fn show_customer(guid) -> {}
pub async fn update_customer(guid) -> {}
pub async fn delete_customer(guid) -> {}
```

#### Database

For right now, I'll just use an in memory data store to share across the route handlers.

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

### Dependencies

As of right now, I know that I will need the following dependencies:

* Warp - A web server framework for Rust
* Tokio - An asynchronous run-time for Rust
* Serde - A de/serialization library for converting JSON to typed data and vice versa.

### Implementation


#### Models

The first thing I want to do is define my customer model and also start adding some structure to the code.

In `main.rs`, add a single line to the file so the file looks like this:

```rust
mod models;

fn main() {
    println!("Hello, world!");
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
