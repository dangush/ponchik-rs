

use google_sheets4 as sheets4;
use sheets4::Sheets;
use serde_json::Value;
use sheets4::oauth2::{self, authenticator::Authenticator};
use sheets4::{api::ValueRange, hyper, hyper_rustls};
use chrono::Utc;
use super::error::{Result, Error};

//http_client.rs
pub fn http_client() -> hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    return hyper::Client::builder().build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .enable_http2()
            .build(),
        );
    }    

// config.rs
pub struct Config {
    pub priv_key: String,
    pub sheet_id: String,
    pub deposit_range_input: String,
    pub deposit_range_output: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            priv_key: String::from("priv_key.json"),
            sheet_id: String::from("1PNWBP6onxP6F0zMVxNE-GeDQiejbbV9i4-eBXbMJYKY"),
            deposit_range_input: String::from("A2:B"),
            deposit_range_output: String::from("A2:B"),
        }
    }
}

// auth.rs
pub async fn auth(
    config: &Config,
    client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
) -> Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    let secret: oauth2::ServiceAccountKey = oauth2::read_service_account_key(&config.priv_key)
        .await
        .expect("secret not found");

    return oauth2::ServiceAccountAuthenticator::with_client(secret, client.clone())
        .build()
        .await
        .expect("could not create an authenticator");
}

pub async fn read_values(
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    config: &Config,
) -> Result<String> {
    let result = hub
        .spreadsheets()
        .values_get(&config.sheet_id, "Current")
        .doit()
        .await;

    match result {
        Ok((_, spreadsheet)) => { 
            let mut val = String::from("");
            for elem in spreadsheet.values.unwrap().into_iter() {
                for s in &elem {
                    val = s.to_string();
                    println!("{}", val)
                }
                println!("   ")
            }
            return Ok(val.as_str().to_string())
        }
        _ => Err(Error::FailedRequest(format!("Couldn't retrieve values from spreadsheet. Try checking sheet_id in config")))
    }
}

pub async fn write_timestamp() -> Result<String> {
    let config = Config::new();
    let client = http_client();
    let auth = auth(&config, client.clone()).await;
    let hub = Sheets::new(client.clone(), auth);

    let mut value_range = ValueRange::default();
    let now = Utc::now().timestamp();
    value_range.values = Some(vec!(vec!(Value::from(now))));    

    let result = hub.spreadsheets()
                        .values_update(value_range, &config.sheet_id, "Current!A1")
                        .value_input_option("RAW") // Set how the input data should be interpreted.
                        .doit()
                        .await;

    match result {
        Ok(_res) => Ok(now.to_string()),
        Err(e) => Err(Error::Sheets(e))
    }  
}

pub async fn write_values(
    values: Vec<Vec<Value>>,
    hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    config: &Config,
) -> Result<()> {

    let mut value_range = ValueRange::default();
    value_range.values = Some(values);    

    let result = hub.spreadsheets()
                        .values_append(value_range, &config.sheet_id, "A1:B1")
                        .value_input_option("RAW") // Set how the input data should be interpreted.
                        .doit()
                        .await;

    match result {
        Ok(_res) => Ok(()),
        Err(e) => Err(Error::Sheets(e))
    }
}


pub async fn read_entry() -> Result<String> {
    let config = Config::new();
    let client = http_client();
    let auth = auth(&config, client.clone()).await;
    let hub = Sheets::new(client.clone(), auth);

    let result = read_values(&hub, &config).await;

    result
}

pub async fn write_entry(values: Vec<Vec<Value>>) -> Result<()> {
    let config = Config::new();
    let client = http_client();
    let auth = auth(&config, client.clone()).await;
    let hub = Sheets::new(client.clone(), auth);

    let result = write_values(values, &hub, &config).await;

    result
}
