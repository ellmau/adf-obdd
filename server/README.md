# Backend for Webservice

This directory contains the backend for <https://adf-bdd.dev> built using actix.rs.

## Usage

For local development run:

- `docker compose up` to run a MongoDB including a web admin interface
- `MONGODB_URI=mongodb://root:example@localhost:27017/ cargo run -F cors_for_local_development -F mock_long_computations` to start the server, connecting it to the MongoDB and allowing CORS from the frontend (running on a separate development server)

The server listens on `localhost:8080`.
The feature flag `-F mock_long_computations` is optional and just mimics longer computation times by using `std::thread::sleep`. This can be helpful to check how the frontend will behave in such cases.
