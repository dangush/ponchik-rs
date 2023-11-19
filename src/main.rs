mod client;
mod error;
mod method;
mod partition;
mod sheetsdb;
mod data;

use serde_json::json;

use client::SlackClient;
use dotenv;
use error::Result;
use sheetsdb::*;
use tokio::fs;
use std::env;

use crate::partition::random_partition;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Arguments incorrectly provided");
    }

    // Get your token at `https://api.slack.com/apps/<your-bot-id>/oauth?`
    // let oauth_token = env::var("SLACK_OAUTH_TOKEN").expect("SLACK_OAUTH_TOKEN not found");
    let oauth_token: String = String::from(dotenv::var("OAUTH_TOKEN").unwrap());
    let client: SlackClient<'_> = SlackClient::from_key(&oauth_token);

    match args[1].as_str() {
        "new-pairings" => {
            // TODO: add size of group / number of groups argument
            match set_up_meetings(dotenv::var("CHANNEL_ID").unwrap().as_str(), client).await {
                Err(e) => println!("Setting up meetings failed: {:?}", e),
                _ => ()
            }
        }
        "midpoint" => {
            match send_midpoint_checkins(client).await {
                Err(e) => println!("Failed to make midpoint checkins: {:?}", e),
                _ => ()
            }
        }
        _ => panic!()
    }
}

async fn set_up_meetings(channel_id: &str, mut client: SlackClient<'_>) -> Result<()> {

    let mut users = client.members_of_channel(channel_id).await?;
    let user_partitions = random_partition(&mut users, 4);

    // TODO: unwrap 
    let blocks_json_str = fs::read_to_string("src/data/introduction_block.json").await.unwrap();

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
        let mut message = blocks_json_str.replace("channel", channel_id);
        message = message.replace("userid1", partition[0].as_str());
        message = message.replace("userid2", partition[1].as_str());

        let blocks: serde_json::Value = serde_json::from_str(&blocks_json_str)?;

        let channel_id = client.start_direct_message(partition).await?;
        // dbg!(&channel_id);

        client.post_message(&channel_id, &message, blocks).await?;
    }

    // println!("total: {:?}", export);
    sheetsdb::write_entry(export).await?;

    Ok(())
}

async fn send_midpoint_checkins(_client: SlackClient<'_>) -> Result<()> {
    // TODO: Read from google sheets db and decide what actions to perform
    let time = read_entry().await?;
    println!("{:?}", time);

    Ok(())
}
