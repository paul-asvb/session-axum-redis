use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    handler::Handler,
    http::{Request, Response, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use repo::{DynSessionRepo, RedisSessionRepo};

mod repo;

#[tokio::main]
async fn main() {
    let session_repo = Arc::new(RedisSessionRepo) as DynSessionRepo;

    let app = Router::new()
        .route("/", get(root))
        .route("/test", get(test))
        .fallback(handler_404.into_service())
        .layer(Extension(session_repo));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, session-axum-handler!"
}

async fn test() -> impl IntoResponse {
    dotenv::dotenv().ok();
    let client = redis::Client::open(env::var("REDIS_URL").unwrap()).unwrap();
    let mut con = client.get_connection().unwrap();
    let _: () = redis::cmd("SET")
        .arg("my_key")
        .arg("42")
        .query(&mut con)
        .unwrap();
    let s: String = redis::cmd("GET").arg("my_key").query(&mut con).unwrap();
    (StatusCode::OK, s)
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::repo::Peer;

    #[test]
    fn test_struct_deserialise() {
        let data = r#"{"peer_id":"myid","offer":{"type":"type1","sdp":"sdp_example"}}"#;
        let _v: Peer = serde_json::from_str(data).unwrap();
    }
    extern crate redis;

    #[tokio::test]
    async fn test_redis_connection() {
        dotenv::dotenv().ok();

        let client = redis::Client::open(env::var("REDIS_URL").unwrap()).unwrap();
        let mut con = client.get_connection().unwrap();
        let _: () = redis::cmd("SET")
            .arg("my_key")
            .arg("42")
            .query(&mut con)
            .unwrap();
        let bar: String = redis::cmd("GET").arg("my_key").query(&mut con).unwrap();
        dbg!(bar);
    }
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
