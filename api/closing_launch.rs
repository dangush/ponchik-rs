use serde::{Deserialize, Serialize};
use vercel_runtime::{run, Body, Error,
    Request, Response, StatusCode,
};

use tracing::instrument;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

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

#[instrument]
pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {

    let _ = ponchik::send_closing_survey().await;

    Ok(Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", "application/json")
    .body(serde_json::json!({"message": "Send midpoint checkins"}).to_string().into())?)
}