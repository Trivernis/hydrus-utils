use hydrus_api::Hydrus;

use crate::error::Result;
use crate::utils::reddit::get_post_images;

#[tracing::instrument(level = "debug", skip(hydrus))]
pub async fn find_and_send_reddit_posts(hydrus: &Hydrus, post_urls: Vec<String>) -> Result<()> {
    let total_posts = post_urls.len();

    for (index, post) in post_urls.into_iter().enumerate() {
        tracing::info!("Importing post {} of {}", index + 1, total_posts);
        if let Err(e) = import_post(&post, hydrus).await {
            tracing::error!("Failed to import {}: {}", post, e);
        }
    }

    Ok(())
}

#[tracing::instrument(level = "debug", skip(hydrus))]
async fn import_post(post_url: &str, hydrus: &Hydrus) -> Result<()> {
    tracing::debug!("Post {}", post_url);
    let images = get_post_images(post_url).await?;
    tracing::info!("Found {} images for post {}", images.len(), post_url);

    for url in images {
        hydrus.import().url(url).run().await?;
    }
    Ok(())
}
