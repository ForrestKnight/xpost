use anyhow::{Context, Result};
use oauth1_request as oauth;
use reqwest::multipart;
use serde::{Deserialize, Serialize};

use crate::config::TwitterConfig;

pub struct TwitterClient {
    config: TwitterConfig,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct TweetResponse {
    pub data: TweetData,
}

#[derive(Debug, Deserialize)]
pub struct TweetData {
    pub id: String,
    #[allow(dead_code)]
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaUploadResponse {
    pub media_id_string: String,
}

#[derive(Debug, Serialize)]
struct TweetRequest {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<MediaIds>,
}

#[derive(Debug, Serialize)]
struct MediaIds {
    media_ids: Vec<String>,
}

impl TwitterClient {
    pub fn new(config: TwitterConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn upload_media(&self, image_data: &[u8]) -> Result<String> {
        let url = "https://upload.twitter.com/1.1/media/upload.json";
        
        // Create OAuth authorization header
        let auth_header = self.create_oauth_header("POST", url, &[]);

        let form = multipart::Form::new()
            .part(
                "media",
                multipart::Part::bytes(image_data.to_vec())
                    .file_name("image.png")
                    .mime_str("image/png")?,
            );

        let response = self.client
            .post(url)
            .header("Authorization", auth_header)
            .multipart(form)
            .send()
            .await
            .context("Failed to upload media")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Media upload failed: {}", error_text);
        }

        let media_response: MediaUploadResponse = response.json().await?;
        Ok(media_response.media_id_string)
    }

    pub async fn post_tweet(&self, text: String, media_id: Option<String>) -> Result<TweetData> {
        let url = "https://api.twitter.com/2/tweets";
        
        let tweet_request = TweetRequest {
            text,
            media: media_id.map(|id| MediaIds {
                media_ids: vec![id],
            }),
        };

        let body = serde_json::to_string(&tweet_request)?;
        
        // Create OAuth authorization header
        let auth_header = self.create_oauth_header("POST", url, &[]);

        let response = self.client
            .post(url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .context("Failed to post tweet")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("API Error {}: {}. Check app permissions at https://developer.twitter.com/en/portal/dashboard", status, error_text);
        }

        let tweet_response: TweetResponse = response.json().await?;
        Ok(tweet_response.data)
    }

    fn create_oauth_header(&self, method: &str, url: &str, _params: &[(&str, &str)]) -> String {
        let client = oauth::Credentials::new(
            &self.config.api_key,
            &self.config.api_secret,
        );

        let token_creds = oauth::Credentials::new(
            &self.config.access_token,
            &self.config.access_token_secret,
        );

        let token = oauth::Token::new(client.clone(), token_creds);

        oauth::authorize(
            method,
            url,
            &(),
            &token,
            oauth::HmacSha1::new(),
        )
    }
}
