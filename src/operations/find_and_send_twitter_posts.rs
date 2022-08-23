use egg_mode::Token;
use hydrus_api::Hydrus;

use crate::config::TwitterConfig;
use crate::error::Result;
use crate::utils::twitter::{get_token, get_tweet_media};

#[tracing::instrument(level = "debug", skip(hydrus))]
pub async fn find_and_send_twitter_posts(
    hydrus: &Hydrus,
    twitter_cfg: TwitterConfig,
    post_urls: Vec<String>,
) -> Result<()> {
    let token = get_token(twitter_cfg).await?;
    let total_posts = post_urls.len();

    for (index, post) in post_urls.into_iter().enumerate() {
        tracing::info!("Importing post {} of {}", index + 1, total_posts);
        if let Err(e) = import_post(&post, hydrus, &token).await {
            tracing::error!("Failed to import {}: {}", post, e);
        }
    }

    Ok(())
}

#[tracing::instrument(level = "debug", skip(hydrus))]
async fn import_post(post_url: &str, hydrus: &Hydrus, token: &Token) -> Result<()> {
    tracing::debug!("Tweet {}", post_url);
    let images = get_tweet_media(post_url, token).await?;
    tracing::info!("Found {} images for tweet {}", images.len(), post_url);

    for url in images {
        hydrus.import().url(url).run().await?;
    }
    Ok(())
}
