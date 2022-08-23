use std::borrow::Cow;

use crate::{config::TwitterConfig, error::Result};
use egg_mode::auth::Token;

/// Returns the token that
/// can be used for twitter api requests
#[tracing::instrument(level = "debug")]
pub async fn get_token(config: TwitterConfig) -> Result<Token> {
    let con_token = egg_mode::KeyPair::new(
        Cow::from(config.consumer_key),
        Cow::from(config.consumer_secret),
    );
    let token = egg_mode::auth::bearer_token(&con_token).await?;

    Ok(token)
}

/// Returns the media urls for a given tweet
#[tracing::instrument(level = "debug", skip(token))]
pub async fn get_tweet_media(url: &str, token: &Token) -> Result<Vec<String>> {
    let id = get_tweet_id(url)?;
    let tweet = egg_mode::tweet::show(id, token).await?.response;

    if let Some(entities) = tweet.extended_entities {
        let media = entities.media;
        let urls: Vec<String> = media
            .into_iter()
            .map(|m| {
                if let Some(video_info) = m.video_info {
                    video_info.variants.into_iter().next().unwrap().url
                } else {
                    m.media_url_https
                }
            })
            .collect();
        Ok(urls)
    } else {
        Ok(Vec::new())
    }
}

/// Returns the tweet ID for a given twitter url
#[tracing::instrument(level = "debug")]
fn get_tweet_id(url: &str) -> Result<u64> {
    let mut url = url;
    if let Some((left, _right)) = url.split_once('?') {
        url = left;
    }
    let id = url
        .rsplit('/')
        .filter(|s| !s.is_empty())
        .next()
        .ok_or("No Tweet ID in twitter url")?;
    let id = id
        .parse::<u64>()
        .map_err(|_| "Tweet ID cannot be parsed as u64")?;

    Ok(id)
}
