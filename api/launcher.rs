use ponchik::{db, client::*};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{event, instrument, span, Level};
use tracing_subscriber;
use vercel_runtime::{
    http::bad_request, run, Body, Error, Request, RequestPayloadExt, Response, StatusCode,
};
use core::fmt;
use std::env;
use stopwatch::{Stopwatch};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    run(handler).await
}

#[derive(Debug, Deserialize, Serialize)]
struct Payload {
    action: String,
    api_key: String,
}

enum Action {
    LaunchIntros,
    LaunchMidpoint,
    LaunchClosing,
    TestGenPairs,
    TestCurRound
}

impl std::str::FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "launch_intro" => Ok(Action::LaunchIntros),
            "launch_midpoint" => Ok(Action::LaunchMidpoint),
            "launch_closing" => Ok(Action::LaunchClosing),
            "test_gen_pairs" => Ok(Action::TestGenPairs),
            "test_cur_round" => Ok(Action::TestCurRound),
            _ => Err(String::from("invalid option")),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct APIError {
    pub message: &'static str,
    pub code: &'static str,
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for APIError {}

#[instrument]
pub async fn handler(req: Request) -> Result<Response<Body>, Error> {

    let payload = match req.payload::<Payload>() {
        Ok(Some(p)) => p,
        Ok(None) => {
            return bad_request(APIError {
                message: "No payload",
                code: "no_payload",
            });
        }
        Err(_) => {
            return bad_request(APIError {
                message: "Invalid payload",
                code: "invalid_payload",
            });
        }
    };

    dotenv::dotenv().ok();
    
    if String::from(env::var("ADMIN_KEY").unwrap()) != payload.api_key {
        // Acknowledge user interaction
        return bad_request(APIError {
            message: "Missing api key",
            code: "no_authorization",
        });
    }

    let action = payload.action.parse::<Action>()
        .map_err(|e| {
            println!("Error parsing action: {}", e);
            e // Consider a more descriptive error handling or propagation strategy
        })?;

    // TODO add error handling 
    let return_json = match action {
        Action::LaunchIntros => {
            let _ = ponchik::set_up_meetings().await;
            
            serde_json::json!({"message": "intros sent successfully"})
        },
        Action::LaunchMidpoint => {
            let _ = ponchik::send_midpoint_checkins().await;
            
            serde_json::json!({"message": "midpoints sent successfully"})
        },
        Action::LaunchClosing => {
            let _ = ponchik::send_closing_survey().await;
            
            serde_json::json!({"message": "closings set up successfully"})
        },
        Action::TestCurRound => {
            let pool = ponchik::db::db_init().await?;
            let cur_round = ponchik::db::db_find_max_round(&pool).await
                .map_err(|e| {
                    println!("Error finding max round: {}", e);
                    e
                })?;
            println!("{}", cur_round);
            
            serde_json::json!({"current_round": cur_round})
        },
        Action::TestGenPairs => {
            // TODO: get rid of this stopwatch after some sanity tests. Need tracing here lol
            let sw = Stopwatch::start_new();
            let result = ponchik::test_partition_building().await.unwrap(); //TODO: remove unwrap

            println!("{:?}", result);
          
            serde_json::json!({"ponchiks": result, "time": sw.elapsed_ms()})
        },
        _ => unimplemented!()
    };

    // Acknowledge user interaction
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(return_json.to_string().into())?)
}