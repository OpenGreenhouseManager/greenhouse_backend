use axum::{ extract::Path, routing::get, Router };
use core::Rectangle;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/:a/:b", get(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3002").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(Path((a, b)): Path<(u32, u32)>) -> String {
    let rec = Rectangle {
        width: a,
        height: b,
    };
    let cf = rec.circumference();
    format!("{cf}")
}
