use std::path::Path;

use axum::{
    http::StatusCode,
    Json,
    Router, routing::{get, post},
};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use app_properties::AppProperties;
use axum::extract::State;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let properties: AppProperties = AppProperties::new();
    let path = properties.get("path");
    let filename_pattern = properties.get("filename_pattern");

    let app_state = AppState {
        path: path.to_string(),
        filename_pattern: filename_pattern.to_string()
    };

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        // `GET /search` goes to `search_in_files`
        .route("/search/:pattern", get(search_in_files))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
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

// the input to our `search_in_files` handler
#[derive(Deserialize)]
struct SearchRequest {
    pattern: String
}

#[derive(Serialize)]
struct SearchResponse {
    pattern: String,
    list: Vec<SearchResult>
}

#[derive(Serialize)]
struct SearchResult {
    path: String
}

#[derive(Clone)]
struct AppState {
    path: String,
    filename_pattern: String
}

async fn search_in_files(State(state): State<AppState>, pattern: String) -> (StatusCode, Json<SearchResponse>) {
    let list = search_pattern_at_path(Path::new(&state.path), &pattern, &state.filename_pattern);

    println!("{}", list.len());

    (StatusCode::CREATED, Json(SearchResponse {
        pattern,
        list
    }))
}

fn search_pattern_at_path(path: &Path, pattern: &String, filename_pattern: &String) -> Vec<SearchResult> {
    let mut results = vec!();
    search_in_dir(&mut results, pattern, path.to_string_lossy().to_string(), &filename_pattern);
    results
}

fn search_in_dir(results: &mut Vec<SearchResult>, pattern: &String, current_dir: String, filename_pattern: &String) {
    for entry in WalkDir::new(current_dir.clone())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(filename_pattern)) {
        let f_name = entry.file_name().to_string_lossy();
        println!("file {}", f_name);
        results.push(SearchResult {
            path: entry.path().to_string_lossy().to_string()
        })
    }
}