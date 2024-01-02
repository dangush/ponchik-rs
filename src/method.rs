#[derive(Debug)]
pub enum Method {
    ListMembersOfChannel,
    OpenDirectMessage,
    PostMessage,
    UserIdentity,
}

impl Into<reqwest::Url> for Method {
    fn into(self) -> reqwest::Url {
        let url = match self {
            Method::ListMembersOfChannel => "https://slack.com/api/conversations.members",
            Method::OpenDirectMessage => "https://slack.com/api/conversations.open",
            Method::PostMessage => "https://slack.com/api/chat.postMessage",
            Method::UserIdentity => "https://slack.com/api/users.profile.get",
        };

        reqwest::Url::parse(url).expect("failed to parse URL")
    }
}
