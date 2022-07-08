use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::net::SocketAddr;

use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
//        .route("/", get(root))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/ab/get", post(ab_get))
        .route("/api/ab", post(ab))
        .route("/api/audit", post(audit))
        .route("/api/currentUser", post(current_user))
        .fallback(post(fallback))
        ;
        

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr: SocketAddr = "0.0.0.0:21114".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize, Debug)]
struct LoginRequest {
    username: String,
    password: String,
    id: String,
    uuid: String,
}

#[derive(Serialize, Debug)]
struct LoginReply {
    user: String,
    access_token: String,
}

#[derive(Deserialize, Debug)]
struct CurrentUserRequest {
    id: String,
    uuid: String,
}

#[derive(Serialize, Debug)]
struct CurrentUserResponse {
    error: bool,
    data: String,
}

#[derive(Deserialize, Debug)]
struct AbRequest {
    data: String,
}

#[derive(Deserialize, Debug)]
struct AuditRequest {
    #[serde(default)]
    #[serde(rename = "Id")]
    id_: String,
    #[serde(default)]
    action: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    ip: String,
    #[serde(default)]
    uuid: String,
}

// {
//    peers: [{id: "abcd", username: "", hostname: "", platform: "", alias: "", tags: ["", "", ...]}, ...],
//    tags: [],
// }

#[derive(Serialize, Debug)]
struct AbPeer {
    id: String,
}

#[derive(Serialize, Debug)]
struct Ab {
    tags: Vec<String>,
    peers: Vec<AbPeer>,
}


#[derive(Serialize, Debug)]
struct AbGetResponse {
    error: bool,
    updated_at: String,
    data: String,
}


async fn login(
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    let reply = LoginReply {
        user: "user".to_string(),
        access_token: "uusseerr".to_string(),
    };

    tracing::debug!("login: {:?}", request);
    (StatusCode::OK, Json(reply))
}

async fn ab_get(
) -> impl IntoResponse {
    tracing::debug!("ab get");

    let reply = AbGetResponse {
        error: false,
        updated_at: "now".to_string(),
        data: serde_json::to_string(&Ab {
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            peers: vec![AbPeer{id: "peer1".to_string()}, AbPeer{id: "513785419".to_string()}],
        }).unwrap()
    };

    tracing::debug!("ab get reply: {:?}", Json(&reply));
    (StatusCode::OK, Json(reply))
}

async fn ab(
    Json(request): Json<AbRequest>,
) -> impl IntoResponse {
    tracing::debug!("ab: {:?}", request);
    StatusCode::OK
}

async fn current_user(
    Json(request): Json<CurrentUserRequest>,
) -> impl IntoResponse {
    let reply = LoginReply {
        user: "user".to_string(),
        access_token: "uusseerr".to_string(),
    };

    tracing::debug!("login: {:?}", request);
    (StatusCode::OK, Json(reply))
}

async fn audit(
    Json(request): Json<AuditRequest>,
) -> impl IntoResponse {
    tracing::debug!("audit: {:?}", request);
    StatusCode::OK
}

async fn logout(
    Json(request): Json<CurrentUserRequest>,
) -> impl IntoResponse {
    tracing::debug!("logout: {:?}", request);
    StatusCode::OK
}

async fn fallback(
    Json(request): Json<CurrentUserRequest>,
) -> impl IntoResponse {
    tracing::debug!("logout: {:?}", request);
    StatusCode::OK
}
