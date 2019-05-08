#[derive(Serialize)]
pub enum ResponseType {
    #[serde(rename = "ephemeral")]
    Ephemeral,

    #[serde(rename = "in_channel")]
    InChannel,
}

impl Default for ResponseType {
    fn default() -> ResponseType {
        ResponseType::Ephemeral
    }
}

#[derive(Serialize, Default)]
pub struct SlackResponse {
    pub response_type: ResponseType,
    pub text: String,
    pub unfurl_links: bool,
}
