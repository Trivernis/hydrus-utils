use hydrus_api::Hydrus;

use crate::error::Result;
use crate::utils::reddit::get_post_images;

pub async fn find_and_send_reddit_posts(hydrus: &Hydrus, post_urls: Vec<String>) -> Result<()> {
    let total_posts = post_urls.len();

    for (index, post) in post_urls.into_iter().enumerate() {
        tracing::info!("Importing post {} of {}", index + 1, total_posts);
        let images = get_post_images(&post).await?;
        tracing::info!("Found {} images for post {}", images.len(), post);

        for url in images {
            hydrus.import().url(url).run().await?;
        }
    }

    Ok(())
}
