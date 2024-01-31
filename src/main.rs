use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};
use lambda_runtime::tower::ServiceBuilder;
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_1::{from_items, to_item};
use serde_dynamo::from_item;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env::set_var;
use std::sync::Arc;
use tower_http::add_extension::AddExtensionLayer;
use uuid::Uuid;

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

#[derive(Serialize, Deserialize)]
struct Pagination {
    key_1: Option<String>,
    key_2: Option<String>,
    todos: Vec<Todo>,
}

impl Pagination {
    // turn response from dynamo db into pagination struct
    fn from_last_evaluated_key(key: &HashMap<String, AttributeValue>) -> Self {
        let mut pagination = Pagination {
            key_1: None,
            key_2: None,
            todos: Vec::new(),
        };

        if let Some(key_1) = key.get(USER_ID_COLUMN) {
            // WARNING this wouldn't work for numeric or binary IDs
            if let Ok(s) = key_1.as_s() {
                pagination.key_1 = Some(s.to_string());
            }
        }

        if let Some(key_2) = key.get(TODO_ID_COLUMN) {
            // WARNING this wouldn't work for numeric or binary IDs
            if let Ok(s) = key_2.as_s() {
                pagination.key_2 = Some(s.to_string());
            }
        }

        pagination
    }
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

fn get_table_name() -> Result<String, JsonResponse> {
    match std::env::var("DYNAMO_TABLE_NAME") {
        Ok(val) => Ok(val),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error.to_string() })),
        )),
    }
}

/// Example on how to return status codes and data from an Axum function
async fn health_check() -> (StatusCode, String) {
    let health = true;
    // put some meaningful health check here
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
async fn create_todo(Extension(client): Extension<Arc<Client>>, todo: Json<Todo>) -> WebResult {
    let client = client.clone();
    let todo = todo.0;

    let item_map = to_item(&todo).unwrap();
    let item_json = serde_json::to_value(&todo).unwrap();
    let table_name = get_table_name()?;

    let response = client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item_map))
        .send()
        .await;

    match response {
        Ok(_output) => Ok((StatusCode::CREATED, Json(item_json))),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error.to_string() })),
        )),
    }
}

/// Deletes a todo from the database
async fn delete_todo(
    Extension(client): Extension<Arc<Client>>,
    Path((userid, todoid)): Path<(Uuid, Uuid)>,
) -> WebResult {
    let client = client.clone();

    // a hash map with the hash key and attribute value of userid, then the range key and attribute value of todoid
    let mut primary_key_map = HashMap::with_capacity(2);
    primary_key_map.insert(
        USER_ID_COLUMN.to_owned(),
        AttributeValue::S(userid.to_string()),
    );
    primary_key_map.insert(
        TODO_ID_COLUMN.to_owned(),
        AttributeValue::S(todoid.to_string()),
    );

    let table_name = get_table_name()?;

    let response = client
        .delete_item()
        .table_name(table_name)
        .set_key(Some(primary_key_map))
        .send()
        .await;

    match response {
        Ok(_output) => Ok((StatusCode::OK, Json(json!({ "deleted": true })))),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error.to_string() })),
        )),
    }
}

/// Updates a todo in the database
async fn update_todo(
    Extension(client): Extension<Arc<Client>>,
    Path((userid, todoid)): Path<(Uuid, Uuid)>,
    todo: Json<Todo>,
) -> WebResult {
    let client = client.clone();
    let todo = todo.0;

    let table_name = get_table_name()?;

    let item_map = to_item(&todo).unwrap();
    let item_json = serde_json::to_value(&todo).unwrap();
    let mut primary_key_map = HashMap::with_capacity(2);
    primary_key_map.insert(USER_ID_COLUMN, AttributeValue::S(userid.to_string()));
    primary_key_map.insert(TODO_ID_COLUMN, AttributeValue::S(todoid.to_string()));

    // let response = client
    //     .update_item()
    //     .table_name(table_name)
    //     .set_item(Some(item_map))
    //     .send()
    //     .await;

    todo!();
}

/// Gets a todo from the database
async fn get_todo(
    Extension(client): Extension<Arc<Client>>,
    Path((userid, todoid)): Path<(Uuid, Uuid)>,
) -> WebResult {
    let client = client.clone();
    let table_name = get_table_name()?;

    let response = client
        .get_item()
        .table_name(table_name)
        .key(USER_ID_COLUMN, AttributeValue::S(userid.to_string()))
        .key(TODO_ID_COLUMN, AttributeValue::S(todoid.to_string()))
        .send()
        .await;

    match response {
        Ok(output) => {
            if let Some(item) = output.item {
                let todo: Todo = from_item(item).unwrap();
                let item_json = serde_json::to_value(&todo).unwrap();
                Ok((StatusCode::OK, Json(item_json)))
            } else {
                Ok((StatusCode::OK, Json(json!(null))))
            }
        }
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error.to_string() })),
        )),
    }
}

async fn get_todo_by_user_id(
    Extension(client): Extension<Arc<Client>>,
    Path(userid): Path<Uuid>,
) -> WebResult {
    let client = client.clone();

    let table_name = get_table_name()?;

    let response = client
        .query()
        .table_name(table_name)
        .key_condition_expression("#user_id = :hashKey")
        .expression_attribute_names("#user_id", USER_ID_COLUMN)
        .expression_attribute_values(":hashKey", AttributeValue::S(userid.to_string()))
        // set set_exclusive_start_key if paginating with the primary key
        .send()
        .await;

    match response {
        Ok(output) => {
            let (items, last_evaluated_key) = (output.items, output.last_evaluated_key);

            match (items, last_evaluated_key) {
                (Some(items), Some(last_evaluated_key)) => {
                    let todos = from_items::<Todo>(items).unwrap();
                    let mut pagination = Pagination::from_last_evaluated_key(&last_evaluated_key);
                    pagination.todos = todos;
                    let items_json = serde_json::to_value(&pagination).unwrap();
                    Ok((StatusCode::OK, Json(items_json)))
                }
                (Some(items), None) => {
                    let todos = from_items::<Todo>(items).unwrap();
                    let items_json = serde_json::to_value(&todos).unwrap();
                    Ok((StatusCode::OK, Json(items_json)))
                }
                (None, Some(last_evaluated_key)) => {
                    // not sure when this would happen
                    let pagination = Pagination::from_last_evaluated_key(&last_evaluated_key);
                    let items_json = serde_json::to_value(&pagination).unwrap();
                    Ok((StatusCode::OK, Json(items_json)))
                }
                (None, None) => Ok((StatusCode::OK, Json(json!([])))),
            }
        }
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error.to_string() })),
        )),
    }
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
        .route("/todo/user/:userid", get(get_todo_by_user_id))
        .route("/todo", post(create_todo))
        .route("/health", get(health_check));

    let app = ServiceBuilder::new()
        .layer(AddExtensionLayer::new(client))
        .service(router);

    lambda_http::run(app).await
}
