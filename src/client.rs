use std::{borrow::Borrow, collections::HashMap};

use reqwest::Url;
use serde_json::Value;
use std::env;

use crate::error::{Error, Result};
use crate::method::Method;

// TODO: Get rid of user_map
// TODO: Actually nevermind lmao get rid of this entire thing, I never reuse my http connections anyway
pub struct SlackClient<'a> {
    api_key: &'a str,
    http_client: reqwest::Client,
    pub users_map: HashMap<String, String>
}

impl<'a> SlackClient<'a> {
    pub fn from_key(api_key: &'a str) -> Self {
        Self {
            api_key,
            http_client: reqwest::Client::new(),
            users_map: HashMap::new()
        }
    }

    // todo: error treatment
    pub async fn send<P, K, V>(&self, method: Method, parameters: P) -> Result<serde_json::Value>
    where
        P: IntoIterator + Send,
        K: AsRef<str>,
        V: AsRef<str>,
        P::Item: Borrow<(K, V)>,
    {
        let mut url: Url = method.into();

        // Adds a sequence of name/value pairs in `application/x-www-form-urlencoded` syntax
        // to the URL
        url.query_pairs_mut().extend_pairs(parameters);

        let response = self
            .http_client
            .post(url)
            .bearer_auth(self.api_key)
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&response)?)
    }

    pub async fn members_of_channel(&mut self, channel: &str) -> Result<Vec<String>> {
        let mut parameters = HashMap::new();
        parameters.insert("channel", channel);

        let response = self.send(Method::ListMembersOfChannel, parameters).await?;

        // Currently pulls the full names of members in a channel. Must be changed back to userids for real implementation
        // TODO: avoid these clone() calls
        // TODO: figure out a way to handle userid mapping that doesn't require burst api calls every time. Maybe store in DB
        let mut userid_array: Vec<String> = response["members"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        userid_array.retain(|user_id| user_id != &env::var("BOT_ID").unwrap());

        // for userid in userid_array.clone() {
        //     let user_name = self.userid_to_identity(&userid).await;
        //     self.users_map.insert(userid.clone(), user_name.clone());
        //     println!("User string: {}", user_name);
        // }

        Ok(userid_array)

        // if userid_array {
        //     Err(Error::FailedRequest(format!("{}", response)))
        // }
    }

    // TODO: Wrap result in a Result<> to error handle
    pub async fn userid_to_identity(&self, user_id: &str) -> String {
        let mut parameters = HashMap::new();
        parameters.insert("user", user_id);

        let response = self.send(Method::UserIdentity, parameters).await.unwrap();

        if matches!(response["ok"], Value::Bool(true)) {
            return response["profile"]["real_name"].to_string();
        } else {
            return String::from("");
        }
    }

    pub async fn start_direct_message(&self, users: &Vec<String>) -> Result<String> {
        let users = users.join(",");
        let mut parameters = HashMap::new();
        parameters.insert("users", &*users);
        parameters.insert("return_im", "false");

        let response = self.send(Method::OpenDirectMessage, parameters).await?;

        if let Value::Object(map) = &response["channel"] {
            let channel_id = map["id"].to_string();
            Ok(channel_id
                .strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap()
                .into())
        } else {
            Err(Error::FailedRequest(format!("{}", response)))
        }
    }

    pub async fn post_message(&self, channel_id: &str, blocks: &Value) -> Result<()> {
        let mut parameters = HashMap::new();
        parameters.insert("channel", channel_id);
        let blocks_str = serde_json::to_string(&blocks).unwrap();
        parameters.insert("blocks", blocks_str.as_str());

        let response = self.send(Method::PostMessage, parameters).await?;

        if matches!(response["ok"], Value::Bool(true)) {
            Ok(())
        } else {
            Err(Error::FailedRequest(response.to_string()))
        }
    }
}
