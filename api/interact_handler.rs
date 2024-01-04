use ponchik::db;
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    run(handler).await
}

#[derive(Debug, Deserialize, Serialize)]
struct Payload {
    payload: String,
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

    let json_value = serde_json::from_str::<Value>(&payload.payload).map_err(|_| APIError {
        message: "could not parse JSON successfully",
        code: "json_parse_error",
    })?;

    if let (Some(response_url), Some(actions)) = (json_value.get("response_url"), json_value.get("actions")) {
        let action_val = actions[0].get("value").unwrap().as_str().unwrap();
        let user_id = json_value["user"]["id"].as_str().unwrap();
        let channel_id = json_value["channel"]["id"].as_str().unwrap();

        event!(Level::DEBUG, "Received JSON: {:?}", json_value);
        handle_action(action_val, user_id, channel_id, response_url.as_str().unwrap()).await
    } else {
        bad_request(APIError {
            message: "Invalid payload structure",
            code: "invalid_structure",
        })
    }
}


async fn handle_action(action: &str, user_id: &str, channel_id: &str, response_url: &str) -> Result<Response<Body>, Error> {
    let mut map = HashMap::new();
    map.insert("replace_original", String::from("true"));

    match action {
        "mid_yes" => {
            map.insert("text", 
            format!("It's time for a midpoint checkin!\n\n *Did you get a chance to meet?*\n\n✅ <@{}> said that you've met!", user_id));
            

            let pool = db::db_init().await?;
            db::db_update_status(&pool, channel_id.to_string(), db::MeetingStatus::Closed(db::FinalStatus::Met)).await?;        

        },
        "mid_no" => {
            map.insert("text", 
            format!("It's time for a midpoint checkin!\n\n *Did you get a chance to meet?*\n\n*:C* <@{}> said that you have not scheduled yet.", user_id));

            // TODO: not sure what to make of this status yet        

        },
        "mid_scheduled" => {
            map.insert("text", 
            format!("It's time for a midpoint checkin!\n\n *Did you get a chance to meet?*\n\n📅 <@{}> said that your meeting is scheduled!", user_id));

            let pool = db::db_init().await?;
            db::db_update_status(&pool, channel_id.to_string(), db::MeetingStatus::Scheduled).await?;  

        },
        "close_yes" => {
            map.insert("text", 
            format!("Checking in! Did you guys get a chance to connect?\n\n🥳<@{}> said that you met! Great!", user_id));

            let pool = db::db_init().await?;
            db::db_update_status(&pool, channel_id.to_string(), db::MeetingStatus::Closed(db::FinalStatus::Met)).await?;  

        },
        "close_no" => {
            map.insert("text", 
            format!("Checking in! Did you guys get a chance to connect?\n\n😶‍🌫️<@{}> said no. Better luck next time!", user_id));
            
            // TODO: Check if it was previously scheduled or not
            let pool = db::db_init().await?;
            db::db_update_status(&pool, channel_id.to_string(), db::MeetingStatus::Closed(db::FinalStatus::Fail)).await?;  

        },
        _ => unreachable!()
    }

    // Respond to user interaction
    let res = reqwest::Client::new().post(response_url)
        .json(&map)
        .send()
        .await?;

    // Acknowledge user interaction
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(res.text().await?.into())
        .unwrap())
}
