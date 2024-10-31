use std::sync::Arc;
use axum::{http::StatusCode, response::{Response, IntoResponse}, routing::get, Extension, Router};
use axum_keycloak_auth::{Url, error::AuthError, instance::KeycloakConfig, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, decode::KeycloakToken, PassthroughMode, expect_role};

pub fn public_router() -> Router {
    Router::new()
        .route("/health", get(health))
}

pub fn protected_router(instance: KeycloakAuthInstance) -> Router {
    Router::new()
        .route("/protected", get(protected))
        .layer(
            KeycloakAuthLayer::<String>::builder()
                .instance(instance)
                .passthrough_mode(PassthroughMode::Block)
                .persist_raw_claims(false)
                .expected_audiences(vec![String::from("account")])
                .required_roles(vec![String::from("administrator")])
                .build(),
        )
}

// You may have multiple routers that you want to see protected by a `KeycloakAuthLayer`.
// You can safely attach new `KeycloakAuthLayer`s to different routers, but consider using only a single `KeycloakAuthInstance` for all of these layers.
// Remember: The `KeycloakAuthInstance` manages the keys used to decode incoming JWTs and dynamically fetches them from your Keycloak server.
// Having multiple instances simultaneously would increase pressure on your Keycloak instance on service startup and unnecessarily store duplicated data.
// The `KeycloakAuthLayer` therefore really takes an `Arc<KeycloakAuthInstance>` in its `instance` method!
// Presence of the `Into` trait in the `instance` methods argument let us hide that fact in the previous example.

#[allow(dead_code)]
pub fn protect(router:Router, instance: Arc<KeycloakAuthInstance>) -> Router {
    router.layer(
        KeycloakAuthLayer::<String>::builder()
            .instance(instance)
            .passthrough_mode(PassthroughMode::Block)
            .persist_raw_claims(false)
            .expected_audiences(vec![String::from("account")])
            .required_roles(vec![String::from("administrator")])
            .build(),
    )
}

// Lets also define the handlers ('health' and 'protected') defined in our routers.

// The `health` handler can always be called without a JWT,
// as we only attached an instance of the `KeycloakAuthLayer` to the protected router.

// The `KeycloakAuthLayer` makes the parsed token data available using axum's `Extension`'s,
// including the users roles, the uuid of the user, its name, email, ...
// The `protected` handler will (in the default `PassthroughMode::Block` case) only be called
// if the request contained a valid JWT which not already expired.
// It may then access that data (as `KeycloakToken<YourRoleType>`) through an Extension
// to get access to the decoded keycloak user information as shown below.

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn protected(Extension(token): Extension<KeycloakToken<String>>) -> Response {
    expect_role!(&token, "administrator");

    tracing::info!("Token payload is {token:#?}");

    (
        StatusCode::OK,
        format!(
            "Hello {name} ({subject}). Your token is valid for another {valid_for} seconds.",
            name = token.extra.profile.preferred_username,
            subject = token.subject,
            valid_for = (token.expires_at - time::OffsetDateTime::now_utc()).whole_seconds()
        ),
    ).into_response()
}

// You can construct a `KeycloakAuthInstance` using a single value of type `KeycloakConfig`, which is constructed using the builder pattern.
// You may want to immediately wrap it inside an `Arc` if you intend to pass it to multiple `KeycloakAuthLayer`s. We are not doing this in this example.

// Your final router can be created by merging the public and protected routers.

#[tokio::main]
async fn main() {
    let keycloak_auth_instance = KeycloakAuthInstance::new(
        KeycloakConfig::builder()
            .server(Url::parse("https://localhost:8443/").unwrap())
            .realm(String::from("MyRealm"))
            .build(),
    );
    let router = public_router().merge(protected_router(keycloak_auth_instance));

    let addr_and_port = String::from("0.0.0.0:8080");
    let socket_addr: std::net::SocketAddr = addr_and_port.parse().unwrap();
    println!("Listening on: {}", addr_and_port);

    let tcp_listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(tcp_listener, router.into_make_service()).await.unwrap();
}