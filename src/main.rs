use ponchik::{set_up_meetings, send_midpoint_checkins};
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Arguments incorrectly provided");
    }

    // Get your token at `https://api.slack.com/apps/<your-bot-id>/oauth?`
    // let oauth_token = env::var("SLACK_OAUTH_TOKEN").expect("SLACK_OAUTH_TOKEN not found");
    // let oauth_token: String = String::from(dotenv::var("OAUTH_TOKEN").unwrap());
    // let client: SlackClient<'_> = SlackClient::from_key(&oauth_token);

    match args[1].as_str() {
        "new-pairings" => {
            // TODO: add size of group / number of groups argument
            match set_up_meetings(env::var("CHANNEL_ID").unwrap().as_str()).await {
                Err(e) => println!("Setting up meetings failed: {:?}", e),
                _ => ()
            }
        }
        "midpoint" => {
            match send_midpoint_checkins().await {
                Err(e) => println!("Failed to make midpoint checkins: {:?}", e),
                _ => ()
            }
        }
        _ => panic!()
    }
}