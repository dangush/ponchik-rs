
use serde_json::json;

use client::SlackClient;
use error::Result;
use std::env;

mod client;
mod error;
mod method;
mod partition;
mod sheetsdb;
mod data;
pub mod db;

use crate::partition::random_partition;

pub async fn set_up_meetings(channel_id: &str) -> Result<()> {
    dotenv::dotenv().ok();
    
    let oauth_token: String = String::from(env::var("OAUTH_TOKEN").unwrap());
    let mut client: SlackClient<'_> = SlackClient::from_key(&oauth_token);

    let mut users = client.members_of_channel(channel_id).await?;
    let user_partitions = random_partition(&mut users, 2);

    // Vector of pairings that will be pushed to Google Sheets 
    let mut export: Vec<Vec<serde_json::Value>> = Vec::default();
    for partition in user_partitions {
        // Convert userid's into full names
        let identity_vec: Vec<serde_json::Value> = partition
                                        .iter()
                                        .filter_map(|userid| Some(json!(client.users_map.get(userid))))
                                        .collect();
                                     
        // println!("Created a DM with {:?}", identity_vec);
        export.push(identity_vec);

        // Create message
        let message = data::message_blocks::INTRO_BLOCK.replace("channel", channel_id)
                                                        .replace("userid1", partition[0].as_str())
                                                        .replace("userid2", partition[1].as_str());

        let blocks: serde_json::Value = serde_json::from_str(&message)?;

        let channel_id = client.start_direct_message(partition).await?;
        // dbg!(&channel_id);

        client.post_message(&channel_id, blocks).await?;
    }

    // println!("total: {:?}", export);
    sheetsdb::write_entry(export).await?;

    Ok(())
}

pub async fn send_midpoint_checkins() -> Result<()> {
    dotenv::dotenv().ok();
    // TODO: read from db

    let oauth_token: String = String::from(env::var("OAUTH_TOKEN").unwrap());
    let client: SlackClient<'_> = SlackClient::from_key(&oauth_token);

    let blocks: serde_json::Value = serde_json::from_str(data::message_blocks::MIDPOINT_BLOCK)?;

    //TODO: change channel_id env variable to a loop for every live pair
    client.post_message(env::var("CHANNEL_ID").unwrap().as_str(), blocks).await?;

    Ok(())
}
