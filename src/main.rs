use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;

mod proxy;

#[tokio::main]
async fn main() {
    // Define your app routes
    let app = Router::new()
        .route("/proxy", get(proxy::handle_proxy));

    // Define socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on http://{}", addr);

    // Run Axum server (uses hyper internally)
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app
    )
    .await
    .unwrap();
}
