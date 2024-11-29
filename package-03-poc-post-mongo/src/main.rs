use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
    extract::{State},
};
use serde::{Deserialize, Serialize};
use mongodb::{ bson::doc, Client, Collection };

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    /*let uri = "mongodb://root:root@localhost:27017/?authSource=admin";
    let client = Client::with_uri_str(uri).await;
    let database = client.database("poc");
    let my_coll: Collection<&User> = database.collection("users");
*/

    let client = create_database_connection().await.unwrap();
    let database = client.database("poc");
    let my_coll: Collection<User> = database.collection("users");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .with_state(my_coll);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    // Ok(()).expect("uh uh");
}

async fn create_database_connection() -> Result<Client, mongodb::error::Error> {
    // dotenv().ok(); //Loading environment variables from .env file
    /*let connection_parameters = mongo_connection::ConnectionString{
        username: "root",
        password: "root",
        cluster: "localhost:27017"
    };*/
    let uri: String = "mongodb://root:root@localhost:27017/?authSource=admin".to_string();
    /*let options = ClientOptions::parse(&url).await?;*/
    return Client::with_uri_str(uri).await;
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(collection): State<Collection<User>>,
    Json(payload): Json<CreateUser>
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    insert(&user, &collection).await;

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

async fn insert(user: &User, collection: &Collection<User>) {
    collection.insert_one(user).await.unwrap();
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
