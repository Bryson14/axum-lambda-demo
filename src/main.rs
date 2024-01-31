use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::{
    body::Body,
    extract::{Path, Request},
    response::Json,
    routing::{get, post},
    Router,
};
use lambda_runtime::tower::ServiceBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env::set_var;
use std::sync::Arc;
use tower_http::add_extension::AddExtensionLayer;
use uuid::Uuid;

const TABLE_NAME: &str = "TODO_TABLE";
const USER_ID_COLUMN: &str = "user_id";
const TODO_ID_COLUMN: &str = "todo_id";

#[derive(Serialize, Deserialize)]
struct Todo {
    // required from client request
    user_id: Uuid,
    #[serde(default = "create_uuid")]
    todo_id: Uuid,
    #[serde(default = "create_blank_title")]
    title: String,
    #[serde(default = "create_state")]
    completed: bool,
}

fn create_uuid() -> Uuid {
    Uuid::now_v7()
}
fn create_blank_title() -> String {
    String::new()
}
fn create_state() -> bool {
    false
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

type WebResult = Result<JsonResponse, JsonResponse>;
type JsonResponse = (StatusCode, Json<Value>);

/// Creates a new todo in the database
async fn create_todo(Extension(client): Extension<Arc<Client>>, req: Body) -> WebResult {
    let client = client.clone();

    todo!();
}

/// Deletes a todo from the database
async fn delete_todo(
    Extension(client): Extension<Arc<Client>>,
    Path((userid, todoid)): Path<(Uuid, Uuid)>,
) -> WebResult {
    let client = client.clone();

    todo!();
}

/// Updates a todo in the database
async fn update_todo(
    Extension(client): Extension<Arc<Client>>,
    Path((userid, todoid)): Path<(Uuid, Uuid)>,
    req: Request<Body>,
) -> WebResult {
    let client = client.clone();

    todo!();
}

/// Gets a todo from the database
async fn get_todo(
    Extension(client): Extension<Arc<Client>>,
    Path((userid, todoid)): Path<(Uuid, Uuid)>,
) -> WebResult {
    let client = client.clone();

    let response = client
        .get_item()
        .table_name("")
        .key(USER_ID_COLUMN, AttributeValue::S(userid.to_string()))
        .key(TODO_ID_COLUMN, AttributeValue::S(todoid.to_string()))
        .send()
        .await;

    todo!();
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
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
    let client: Arc<Client> = Arc::new(Client::new(&config));

    let router: axum::Router = Router::new()
        .route(
            "/todo/user/:userid/id/:todoid",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .route("/todo", post(create_todo))
        .route("/health", get(health_check));

    let app = ServiceBuilder::new()
        .layer(AddExtensionLayer::new(client))
        .service(router);

    lambda_http::run(app).await
}
