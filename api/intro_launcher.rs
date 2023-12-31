use serde::{Deserialize, Serialize};
use vercel_runtime::{run, Body, Error,
    Request, Response, StatusCode,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[derive(Debug, Deserialize, Serialize)]
struct Payload {
    payload: String,
}

#[derive(Serialize)]
pub struct APIError {
    pub message: &'static str,
    pub code: &'static str,
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {

    ponchik::set_up_meetings().await;

    Ok(Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", "application/json")
    .body(serde_json::json!({"message": "Send midpoint checkins"}).to_string().into())?)
}