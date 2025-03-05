use std::sync::Arc;
use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use base::RespMessage;
use crate::pr_context::PrContext;
use crate::pr_device_handler::PrDeviceHandler;

pub struct PrServer {
    pub host: String,
    pub port: i32,
    pub context: Arc<tokio::sync::Mutex<PrContext>>,
}

impl PrServer {

    pub fn new(host: String, port: i32, context: Arc<tokio::sync::Mutex<PrContext>>) -> PrServer {
        PrServer {
            host,
            port,
            context
        }
    }

    pub async fn start(&self) {
        // build our application with a route
        let app = Router::new()
            // `GET /` goes to `root`
            .route("/", get(PrServer::root))
            .route("/create/new/device", post(PrDeviceHandler::create_new_device))
            .route("/query/devices", get(PrDeviceHandler::query_devices))
            // `POST /users` goes to `create_user`
            .route("/users", post(create_user))
            .with_state(self.context.clone());

        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:20581").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    pub async fn root(State(ctx): State<Arc<tokio::sync::Mutex<PrContext>>>) -> &'static str {
        "Hello, World!"
    }

}
 

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}