use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use mongodb::{ bson::doc, Client, Collection };

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(create_user: web::Json<CreateUser>, data: web::Data<AppState>) -> impl Responder {
    println!("{}", create_user.username);
    insert(&create_user, &data.collection).await;
    HttpResponse::Ok().body(create_user.username.clone())
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = create_database_connection().await.unwrap();
    let database = client.database("poc");
    let collection: Collection<CreateUser> = database.collection("users");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
                collection: collection.clone()
            }))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(Deserialize, Serialize)]
struct CreateUser {
    username: String,
}

struct AppState {
    app_name: String,
    collection: Collection<CreateUser>
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

async fn insert(user: &CreateUser, collection: &Collection<CreateUser>) {
    collection.insert_one(user).await.unwrap();
}
