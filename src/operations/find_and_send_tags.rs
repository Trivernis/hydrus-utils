use crate::{
    error::Result,
    utils::pixiv::{get_pixiv_url, get_sauces_for_file, get_tags_for_sauce},
};
use hydrus_api::wrapper::{hydrus_file::HydrusFile, service::ServiceName};
use pixiv_rs::PixivClient;
use rustnao::{Handler, Sauce};
use tempdir::TempDir;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn find_and_send_tags(
    finish_tag: Option<&String>,
    handler: &Handler,
    pixiv: &PixivClient,
    service: &ServiceName,
    tmpdir: &TempDir,
    mut file: &mut HydrusFile,
) -> Result<()> {
    if let Err(e) = search_and_assign_tags(&handler, &pixiv, &service, &tmpdir, &mut file).await {
        let hash = file.hash().await.unwrap();
        tracing::error!("Failed to search tags to file {}: {:?}", hash, e);
    } else if let Some(finish_tag) = finish_tag {
        file.add_tags(service.clone().into(), vec![finish_tag.into()])
            .await
            .unwrap();
    }

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn search_and_assign_tags(
    handler: &Handler,
    pixiv: &PixivClient,
    service: &ServiceName,
    tmpdir: &TempDir,
    mut file: &mut HydrusFile,
) -> Result<()> {
    tracing::debug!("Getting tags for hydrus file {:?}", file.id);
    let sauces = get_sauces_for_file(&handler, tmpdir, file).await?;

    assign_pixiv_tags_and_url(&pixiv, service, &mut file, &sauces).await
}

#[tracing::instrument(level = "debug", skip_all)]
async fn assign_pixiv_tags_and_url(
    pixiv: &&PixivClient,
    service: &ServiceName,
    file: &mut &mut HydrusFile,
    sauce: &Vec<Sauce>,
) -> Result<()> {
    let hash = file.hash().await?;
    if let Some(url) = get_pixiv_url(&sauce) {
        let tags = get_tags_for_sauce(&pixiv, url).await?;

        if tags.len() > 0 {
            tracing::info!("Found {} tags for file {:?}", tags.len(), hash);
            file.add_tags(service.clone().into(), tags).await?;
        } else {
            tracing::info!("No tags for file {:?} found", hash);
        }
        file.associate_urls(vec![url.to_string()]).await?;
    } else {
        tracing::info!("No pixiv post for file {:?} found", hash);
    }

    Ok(())
}
