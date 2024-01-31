use aws_config::BehaviorVersion;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use serde::{Deserialize, Serialize};
use axum::http::StatusCode;
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::env::set_var;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Todo {
    todo_id: Uuid,
    title: String,
    completed: bool,
}

impl Default for Todo {
    fn default() -> Self {
        Self {
            todo_id: Uuid::new_v4(),
            title: String::new(),
            completed: false,
        }
    }
}

async fn get_todo_handler(Path(id): Path<String>) -> Result<Json<Todo>, StatusCode> {
    let todo = Todo {
        id: Uuid:XXXXXXX(),
        title: "test".to_string(),
        completed: false,
    };

    Ok(Json(todo))
}




/// Example on how to return status codes and data from an Axum function
async fn health_check() -> (StatusCode, String) {
    let health = true;

    match health {
        true => (StatusCode::OK, "Healthy!".to_string()),
        false => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Not healthy!".to_string(),
        ),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // AWS Runtime can ignore Stage Name passed from json event
    // Remove if you want the first section of the url to be the stage name of the API Gateway
    // i.e with: `GET /test-stage/todo/id/123` without: `GET /todo/id/123`
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let app = Router::new()
        .route("/", get(root))
        .route("/todo/:id", get(get_foo).put(post_foo))
        .route("/foo/:name", post(post_foo_name))
        .route("/health", get(health_check));

    run(app).await
}