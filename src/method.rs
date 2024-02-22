#[derive(Debug)]
pub enum Method {
    ListMembersOfChannel,
    OpenDirectMessage,
    PostMessage,
    UserIdentity,
}

impl From<Method> for reqwest::Url {
    fn from(val: Method) -> Self {
        let url = match val {
            Method::ListMembersOfChannel => "https://slack.com/api/conversations.members",
            Method::OpenDirectMessage => "https://slack.com/api/conversations.open",
            Method::PostMessage => "https://slack.com/api/chat.postMessage",
            Method::UserIdentity => "https://slack.com/api/users.profile.get",
        };

        reqwest::Url::parse(url).expect("failed to parse URL")
    }
}
