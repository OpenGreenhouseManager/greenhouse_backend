use axum::{extract::Path, routing::get, Router};

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/:a/:b", get(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(Path((a, b)): Path<(i32, i32)>) -> String {
    //let area = a * b;
    //let cf = 2 * a + 2 * b;
    let area_request = reqwest::get(format!("http://127.0.0.1:3001/{a}/{b}"));
    let cf_request = reqwest::get(format!("http://127.0.0.1:3002/{a}/{b}"));
    let area =match area_request.await{
        Ok(response) => match response.text().await{
            Ok(text) => text,
            Err(e) => e.to_string()
        }
        Err(e) => e.to_string()
    };
    let cf =match cf_request.await{
        Ok(response) => match response.text().await{
            Ok(text) => text,
            Err(e) => e.to_string()
        }
        Err(e) => e.to_string()
    };

    format!(
        "
        <h1>Rectangle Info</h1>
        <p>Area: {area}</p>
        <p>Circumference: {cf}</p>
    ")
}