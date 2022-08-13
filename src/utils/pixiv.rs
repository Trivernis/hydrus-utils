use crate::error::Result;
use hydrus_api::wrapper::hydrus_file::HydrusFile;
use hydrus_api::wrapper::tag::Tag;
use pixiv_rs::PixivClient;
use rustnao::{Handler, Sauce};
use std::fs;
use std::path::PathBuf;
use tempdir::TempDir;

pub async fn get_sauces_for_file(
    handler: &Handler,
    tmpdir: &TempDir,
    mut file: &mut HydrusFile,
) -> Result<Vec<Sauce>> {
    tracing::debug!("Creating tmp file for hydrus file {:?}", file.id);
    let path = create_tmp_sauce_file(&tmpdir, &mut file).await?;
    tracing::debug!("Getting sauce for hydrus file {:?}", file.id);

    let sauce = handler.get_sauce(path.to_str().unwrap(), None, None)?;
    tracing::debug!("Getting tags for hydrus file {:?}", file.id);
    Ok(sauce)
}

pub fn get_urls(sauce: &Vec<Sauce>) -> Vec<&String> {
    sauce.iter().flat_map(|s| &s.ext_urls).collect()
}

pub async fn get_tags_for_sauce(pixiv: &PixivClient, url: &String) -> crate::Result<Vec<Tag>> {
    let mut tags = Vec::new();

    if let Some(pixiv_id) = url.rsplit_once("=").map(|s| s.1) {
        tracing::trace!("Pixiv id is '{}'", pixiv_id);
        let illustration = pixiv.illustration(pixiv_id).await?;

        for tag in illustration.tags.tags {
            let tag_value = tag.translation.get("en").unwrap_or(&tag.tag);
            tags.push(Tag::from(tag_value));
        }
    }

    Ok(tags)
}

async fn create_tmp_sauce_file(tmpdir: &TempDir, file: &mut HydrusFile) -> crate::Result<PathBuf> {
    let hash = file.hash().await?;
    let bytes = file.retrieve().await?.bytes;
    let path = tmpdir.path().join(&hash);
    fs::write(&path, bytes)?;

    Ok(path)
}

pub fn get_pixiv_url(sauce: &Vec<Sauce>) -> Option<&String> {
    sauce.first().and_then(|s| s.ext_urls.first())
}
