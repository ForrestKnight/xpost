use anyhow::{Context, Result};
use oauth1_request as oauth;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

#[derive(Debug, Deserialize, Clone)]
pub struct UserTweetsResponse {
    pub data: Option<Vec<Tweet>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_metrics: Option<PublicMetrics>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PublicMetrics {
    pub retweet_count: u32,
    pub reply_count: u32,
    pub like_count: u32,
    pub quote_count: u32,
    #[serde(default)]
    pub impression_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct TweetDetailResponse {
    pub data: Tweet,
}

#[derive(Debug, Deserialize)]
pub struct UserMeResponse {
    pub data: UserData,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub data: Option<Vec<Tweet>>,
    pub meta: SearchMeta,
}

#[derive(Debug, Deserialize)]
pub struct SearchMeta {
    pub result_count: usize,
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

    pub async fn get_current_user(&self) -> Result<UserData> {
        let url = "https://api.twitter.com/2/users/me";
        let auth_header = self.create_oauth_header_for_url("GET", url);

        let response = self.client
            .get(url)
            .header("Authorization", auth_header)
            .send()
            .await
            .context("Failed to get current user")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get user: {}", error_text);
        }

        let user_response: UserMeResponse = response.json().await?;
        Ok(user_response.data)
    }

    pub async fn get_user_tweets(&self, user_id: &str, max_results: u32) -> Result<Vec<Tweet>> {
        let url = format!(
            "https://api.twitter.com/2/users/{}/tweets?max_results={}&tweet.fields=created_at,public_metrics",
            user_id, max_results
        );
        let auth_header = self.create_oauth_header_for_url("GET", &url);

        let response = self.client
            .get(&url)
            .header("Authorization", auth_header)
            .send()
            .await
            .context("Failed to get user tweets")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get tweets: {}", error_text);
        }

        let tweets_response: UserTweetsResponse = response.json().await?;
        Ok(tweets_response.data.unwrap_or_default())
    }

    pub async fn get_tweet_details(&self, tweet_id: &str) -> Result<Tweet> {
        let url = format!(
            "https://api.twitter.com/2/tweets/{}?tweet.fields=created_at,public_metrics",
            tweet_id
        );
        let auth_header = self.create_oauth_header_for_url("GET", &url);

        let response = self.client
            .get(&url)
            .header("Authorization", auth_header)
            .send()
            .await
            .context("Failed to get tweet details")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get tweet details: {}", error_text);
        }

        let tweet_response: TweetDetailResponse = response.json().await?;
        Ok(tweet_response.data)
    }

    pub async fn get_tweet_replies(&self, tweet_id: &str, max_results: u32) -> Result<Vec<Tweet>> {
        let url = format!(
            "https://api.twitter.com/2/tweets/search/recent?query=conversation_id:{}&max_results={}&tweet.fields=created_at,author_id",
            tweet_id, max_results.min(100)
        );
        let auth_header = self.create_oauth_header_for_url("GET", &url);

        let response = self.client
            .get(&url)
            .header("Authorization", auth_header)
            .send()
            .await
            .context("Failed to get tweet replies")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get replies: {}", error_text);
        }

        let search_response: SearchResponse = response.json().await?;
        Ok(search_response.data.unwrap_or_default())
    }

    fn create_oauth_header_for_url(&self, method: &str, url: &str) -> String {
        let client = oauth::Credentials::new(
            &self.config.api_key,
            &self.config.api_secret,
        );

        let token_creds = oauth::Credentials::new(
            &self.config.access_token,
            &self.config.access_token_secret,
        );

        let token = oauth::Token::new(client.clone(), token_creds);

        // Parse URL to extract query parameters
        if let Some(query_start) = url.find('?') {
            let base_url = &url[..query_start];
            let query_string = &url[query_start + 1..];
            
            // Parse query parameters into a BTreeMap (automatically sorted)
            let params: BTreeMap<String, String> = query_string
                .split('&')
                .filter_map(|param| {
                    let mut parts = param.splitn(2, '=');
                    match (parts.next(), parts.next()) {
                        (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                        _ => None,
                    }
                })
                .collect();
            
            let request = oauth::request::AssertSorted::new(&params);
            
            oauth::authorize(
                method,
                base_url,
                &request,
                &token,
                oauth::HmacSha1::new(),
            )
        } else {
            oauth::authorize(
                method,
                url,
                &(),
                &token,
                oauth::HmacSha1::new(),
            )
        }
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
