use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    pub ok: bool,
    pub profile: Profile,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    title: String,
    phone: String,
    skype: String,
    pub real_name: String,
    real_name_normalized: String,
    pub display_name: String,
    display_name_normalized: String,
    fields: HashMap<String, ProfileField>,
    status_text: String,
    status_emoji: String,
    status_emoji_display_info: Vec<String>,  // Assuming this is an array of strings, update if needed.
    status_expiration: i64,
    avatar_hash: String,
    start_date: String,
    email: String,
    pronouns: String,
    huddle_state: String,
    huddle_state_expiration_ts: i64,
    first_name: String,
    last_name: String,
    #[serde(rename = "image_24")]
    image24: String,
    #[serde(rename = "image_32")]
    image32: String,
    #[serde(rename = "image_48")]
    image48: String,
    #[serde(rename = "image_72")]
    image72: String,
    #[serde(rename = "image_192")]
    image192: String,
    #[serde(rename = "image_512")]
    image512: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProfileField {
    value: String,
    alt: String,
}
