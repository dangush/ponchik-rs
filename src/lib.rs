
use data::user_profile;
use serde_json::json;

use client::SlackClient;
use error::Result;
use std::env;
use chrono::{Local, Date, Utc};

use tracing_subscriber;
use tracing::{event, span, Level, instrument};

pub mod client;
mod error;
mod method;
mod partition;
mod data;
pub mod db;

use crate::partition::random_partition;

#[instrument]
pub async fn set_up_meetings() -> Result<()> {
    dotenv::dotenv().ok();
    
    let oauth_token: String = String::from(env::var("OAUTH_TOKEN").unwrap());
    let mut client: SlackClient<'_> = SlackClient::from_key(&oauth_token);

    let mut users = client.members_of_channel(env::var("CHANNEL_ID").unwrap().as_str()).await?;
    let user_partitions = random_partition(&mut users, 2);

    // TODO: fix error handling
    let db_pool = db::db_init().await.unwrap();
    let cur_round = db::db_find_max_round(&db_pool).await.unwrap() + 1;

    for partition in user_partitions {        
        event!(Level::INFO, "{:?}", partition);
        // Convert userid's into full names
        // let identity_vec: Vec<serde_json::Value> = partition
        //                                 .iter()
        //                                 .filter_map(|userid| Some(json!(client.users_map.get(userid))))
        //                                 .collect();
                                     
        // Create message
        let message = data::message_blocks::INTRO_BLOCK.replace("channel", env::var("CHANNEL_ID").unwrap().as_str())
                                                        .replace("userid1", partition[0].as_str())
                                                        .replace("userid2", partition[1].as_str());

        let blocks: serde_json::Value = serde_json::from_str(&message)?;

        let channel_id = client.start_direct_message(&partition).await?;

        let pairing = db::Pairing {
            group_channel_id: channel_id.clone(),
            round: cur_round,
            meeting_status: db::MeetingStatus::Open,
            names: partition
        };
        println!("{:?}", pairing);
        // TODO: fix error handling
        let _ = db::db_insert_single(&db_pool, pairing).await;

        client.post_message(&channel_id, &blocks).await?;
    }

    Ok(())
}

#[instrument]
pub async fn send_midpoint_checkins() -> Result<()> {
    dotenv::dotenv().ok();

    // TODO: fix error handling
    let db_pool = db::db_init().await.unwrap();
    let open_pairs = db::db_find_all_status(&db_pool, db::MeetingStatus::Open).await.unwrap();

    let oauth_token: String = String::from(env::var("OAUTH_TOKEN").unwrap());
    let client: SlackClient<'_> = SlackClient::from_key(&oauth_token);

    let blocks: serde_json::Value = serde_json::from_str(data::message_blocks::MIDPOINT_BLOCK)?;

    for pair in open_pairs {
        client.post_message(&pair.group_channel_id, &blocks).await?;
    }

    Ok(())
}

#[instrument]
pub async fn send_closing_survey() -> Result<()> {
    dotenv::dotenv().ok();

    // TODO: fix error handling
    let db_pool = db::db_init().await.unwrap();
    let pairs_to_close = db::db_find_all_status2(&db_pool, db::MeetingStatus::Open, db::MeetingStatus::Scheduled).await.unwrap();

    let oauth_token: String = String::from(env::var("OAUTH_TOKEN").unwrap());
    let client: SlackClient<'_> = SlackClient::from_key(&oauth_token);

    let blocks: serde_json::Value = serde_json::from_str(data::message_blocks::CLOSING_BLOCK)?;

    for pair in pairs_to_close {
        client.post_message(&pair.group_channel_id, &blocks).await?;
    }

    Ok(())
}

pub async fn test_partition_building() -> Result<Vec<Vec<String>>> {
    dotenv::dotenv().ok();
    
    let oauth_token: String = String::from(env::var("OAUTH_TOKEN").unwrap());
    let mut client: SlackClient<'_> = SlackClient::from_key(&oauth_token);

    let mut users = client.members_of_channel(env::var("CHANNEL_ID").unwrap().as_str()).await?;
    let user_partitions = random_partition(&mut users, 4);
    let mut user_partitions_identified: Vec<Vec<String>> = vec![];

    for partition in user_partitions {        
        event!(Level::INFO, "{:?}", partition);
        // Convert userid's into full names
        let mut identity_vec: Vec<String> = vec![];

        for user_id in partition {
            identity_vec.push(client.userid_to_identity(&user_id).await);
        }        
                                     
        println!("{:?}", identity_vec);
        user_partitions_identified.push(identity_vec);
    }

    Ok(user_partitions_identified)
}

