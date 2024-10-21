
### Overview
This is a toy application for handling orders in an imaginary restaurant chain!

It allows waitstaff to create, list, and delete customers' orders using their wearable devices.
The application's API is REST-like and is well-suited for use in environments with poor network connections.

Two major features are intentionally left out of scope:

* Persistence (DB layer)
* Authentication and authorization

However, both features could quite easily be added to the existing application.
* The persistence layer could be added by writing a DB adapter satisfying the OrderService trait.
* Authentication and authorization could be plugged in via Rocket middleware.

See in-code comments for API handlers `main.rs` for more details on API.
### How to run
```
cargo build
APP_PORT=8080 APP_NUM_THREADS=20 cargo run
```

### How to test
```
cargo test
```

### License
MIT
