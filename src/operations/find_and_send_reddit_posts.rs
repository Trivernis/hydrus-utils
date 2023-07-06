use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use hydrus_api::Hydrus;

use crate::error::Result;
use crate::utils::reddit::get_post_images;
use futures::future;

#[tracing::instrument(level = "debug", skip(hydrus))]
pub async fn find_and_send_reddit_posts(hydrus: &Hydrus, post_urls: Vec<String>) -> Result<()> {
    let total_posts = post_urls.len();
    let mut posts_with_img = Vec::new();

    tracing::info!("Retrieving post data...");
    let counter = Arc::new(AtomicUsize::new(1));

    let post_results = future::join_all(post_urls.into_iter().enumerate().map(|(i, p)| {
        let counter = Arc::clone(&counter);

        async move {
            let img = get_post_images(&p).await?;
            tracing::info!(
                "Got info for {} of {total_posts}",
                counter.fetch_add(1, Ordering::SeqCst)
            );

            Result::Ok((i, p, img))
        }
    }))
    .await;

    for result in post_results {
        match result {
            Ok(e) => {
                posts_with_img.push(e);
            }
            Err(e) => {
                tracing::error!("Failed to retrieve post info: {e}");
            }
        }
    }

    for (index, post, images) in posts_with_img {
        tracing::info!("Importing post {} of {}", index + 1, total_posts);
        if let Err(e) = import_post(hydrus, &post, images).await {
            tracing::error!("Failed to import post {}: {}", post, e);
        }
    }

    Ok(())
}

async fn import_post(hydrus: &Hydrus, post: &String, images: Vec<String>) -> Result<()> {
    for url in images {
        let mut entry = hydrus.import().url(url).run().await?;
        let files = entry.files().await?;

        for mut file in files {
            file.associate_urls(vec![post.to_string()]).await?;
        }
    }

    Ok(())
}
