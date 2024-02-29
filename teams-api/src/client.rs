use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::{header, Method, RequestBuilder};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    error::{Error, Result},
    models::{requests::*, responses::*, Activity},
};

#[derive(Clone)]
pub struct TeamsBotClient {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    token: Arc<Mutex<Option<Token>>>,
}

#[derive(Deserialize, Debug)]
struct Token {
    expires_in: usize,
    access_token: String,
    #[serde(skip, default = "Instant::now")]
    acquired: Instant,
}

impl Token {
    fn is_valid(&self) -> bool {
        let elapsed = self.acquired.elapsed();
        elapsed
            < Duration::from_secs(self.expires_in as u64)
                .checked_sub(Duration::from_secs(60))
                .unwrap_or_default()
    }
}

impl TeamsBotClient {
    pub fn new(client: reqwest::Client, client_id: &str, client_secret: &str) -> Self {
        Self {
            client,
            client_id: client_id.to_owned(),
            client_secret: client_secret.to_owned(),
            token: Arc::new(Mutex::new(None)),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_token(&self) -> Result<Token> {
        let data = format!("grant_type=client_credentials&client_id={client_id}&client_secret={client_secret}&scope=https%3A%2F%2Fapi.botframework.com%2F.default", client_id = self.client_id, client_secret = self.client_secret);

        let result = self
            .client
            .post("https://login.microsoftonline.com/botframework.com/oauth2/v2.0/token")
            .header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/x-www-form-urlencoded"),
            )
            .body(data)
            .send()
            .await?;

        match result.status().is_success() {
            false => Err(Error::Teams(result.json().await?)),
            true => Ok(result.json().await?),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn create_request(
        &self,
        method: Method,
        base_url: Option<&str>,
        url: &str,
    ) -> Result<RequestBuilder> {
        let mut token = self.token.lock().await;

        match *token {
            Some(ref t) if !t.is_valid() => *token = Some(self.fetch_token().await?),
            None => *token = Some(self.fetch_token().await?),
            _ => (),
        }

        let request = self
            .client
            .request(
                method,
                format!(
                    "{base_url}{url}",
                    base_url = base_url
                        .map(|x| x.trim_end_matches('/'))
                        .unwrap_or("https://smba.trafficmanager.net/teams")
                ),
            )
            .bearer_auth(&token.as_ref().unwrap().access_token);

        Ok(request)
    }

    /// Creates a new conversation.
    #[tracing::instrument(skip(self, body))]
    pub async fn create_conversation(
        &self,
        base_url: Option<&str>,
        body: &ConversationParameters,
    ) -> Result<ConversationResourceResponse> {
        let result = self
            .create_request(Method::POST, base_url, "/v3/conversations")
            .await?
            .json(body)
            .send()
            .await?;

        match result.status().is_success() {
            false => Err(Error::Teams(result.json().await?)),
            true => Ok(result.json().await?),
        }
    }

    /// Sends an activity (message) to the specified conversation. The activity will be appended to the end of the conversation according to the timestamp or semantics of the channel. To reply to a specific message within the conversation, use Reply to Activity instead.
    #[tracing::instrument(skip(self, body))]
    pub async fn send_to_conversation(
        &self,
        base_url: Option<&str>,
        conversation_id: &str,
        body: &Activity,
    ) -> Result<ResourceResponse> {
        let result = self
            .create_request(
                Method::POST,
                base_url,
                &format!("/v3/conversations/{conversation_id}/activities"),
            )
            .await?
            .json(body)
            .send()
            .await?;

        match result.status().is_success() {
            false => Err(Error::Teams(result.json().await?)),
            true => Ok(result.json().await?),
        }
    }

    /// Some channels allow you to edit an existing activity to reflect the new state of a bot conversation. For example, you might remove buttons from a message in the conversation after the user has clicked one of the buttons. If successful, this operation updates the specified activity within the specified conversation.
    #[tracing::instrument(skip(self, body))]
    pub async fn update_activity(
        &self,
        base_url: Option<&str>,
        conversation_id: &str,
        activity_id: &str,
        body: &Activity,
    ) -> Result<ResourceResponse> {
        let result = self
            .create_request(
                Method::PUT,
                base_url,
                &format!("/v3/conversations/{conversation_id}/activities/{activity_id}"),
            )
            .await?
            .json(body)
            .send()
            .await?;

        match result.status().is_success() {
            false => Err(Error::Teams(result.json().await?)),
            true => Ok(result.json().await?),
        }
    }
}
