# Rust Warp API Example

## Overview

This is an example project that uses Warp and Tokio to build a simple asynchronous api.

## Goals

1. Become familiar with the Warp framework.
2. Get a better understanding of API design in Rust

## Notes

### Design

### Endpoints

```
/customers
    - GET -> list all customers in data store
    - POST -> create new customer and insert into data store
/customers/{guid}
    - GET -> list info for a customer
    - POST -> update information for a customer
    - DELETE -> remove customer from data store
```

### Handlers

Based on the defined endpoints, I will need the following handlers:

> These are just function stubs
```rust
pub async fn list_customers() -> {}
pub async fn create_customer() -> {}
pub async fn show_customer(guid) -> {}
pub async fn update_customer(guid) -> {}
pub async fn delete_customer(guid) -> {}
```

### Database

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