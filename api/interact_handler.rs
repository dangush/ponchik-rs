use serde_json::Value;
use serde::{Deserialize, Serialize};
use vercel_runtime::{
    http::bad_request, run, Body, Error,
    Request, RequestPayloadExt, Response, StatusCode,
};
use reqwest;
use std::collections::HashMap;

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

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {

    let payload = req.payload::<Payload>();

    match payload {
        Ok(Some(payload)) => {
            match serde_json::from_str::<Value>(&payload.payload) {
                Ok(json_value) => {
                    println!("Received JSON: {:?}", json_value);
                    
                    if let Some(response_url) = json_value.get("response_url") {
                        println!("Response url: {}", response_url);
                        if let Some(actions) = json_value.get("actions") {
                            let action_val = actions[0].get("value").unwrap();
                            let user_id = &json_value["user"]["id"].as_str().unwrap();
                            println!("ACTION VALUE: {}", action_val);
                            
                            let mut map = HashMap::new();
                            map.insert("replace_original", String::from("true"));

                            match action_val.as_str().unwrap() {
                                "yes" => {
                                    map.insert("text", 
                                    format!("It's time for a midpoint checkin!\n\n *Did you get a chance to meet?*\n\n
                                    ✅ <@{}> said that you've met!", user_id));
        
                                    let _res = reqwest::Client::new().post(response_url.as_str().unwrap())
                                        .json(&map)
                                        .send()
                                        .await?;        
                                },
                                "no" => {
                                    map.insert("text", 
                                    format!("It's time for a midpoint checkin!\n\n *Did you get a chance to meet?*\n\n
                                    *:C* <@{}> said that you have not scheduled yet.", user_id));
        
                                    let _res = reqwest::Client::new().post(response_url.as_str().unwrap())
                                        .json(&map)
                                        .send()
                                        .await?;        

                                },
                                "scheduled" => {
                                    map.insert("text", 
                                    format!("It's time for a midpoint checkin!\n\n *Did you get a chance to meet?*\n\n 
                                    📅 <@{}> said that your meeting is scheduled!", user_id));
        
                                    let _res = reqwest::Client::new().post(response_url.as_str().unwrap())
                                        .json(&map)
                                        .send()
                                        .await?;  

                                },
                                _ => unreachable!()
                            }
                        }
                    } else {
                        bad_request(APIError {
                            message: "Invalid payload",
                            code: "invalid_payload",
                        })?;
                    }
                                        

                    Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(json_value.to_string().into())
                    .unwrap())
                },
                Err(_) => {
                    Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(serde_json::json!({"message": "could not parse JSON successfully"}).to_string().into())
                    .unwrap()) 
                }
            }
        }
        Err(..) => bad_request(APIError {
            message: "Invalid payload",
            code: "invalid_payload",
        }),
        Ok(None) => bad_request(APIError {
            message: "No payload",
            code: "no_payload",
        }),
    }

}