mod args;
mod error;
pub mod search;

use crate::error::Result;
use crate::search::get_urls;
use args::*;
use clap::Parser;
use hydrus_api::wrapper::hydrus_file::HydrusFile;
use hydrus_api::wrapper::service::ServiceName;
use hydrus_api::wrapper::tag::Tag;
use hydrus_api::{Client, Hydrus};
use pixiv_rs::PixivClient;
use rustnao::{Handler, HandlerBuilder, Sauce};
use tempdir::TempDir;
use tokio::time::{Duration, Instant};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::builder().init();
    let args: Args = Args::parse();
    log::debug!("args: {args:?}");
    let opt = match &args.subcommand {
        Command::FindAndSendUrl(opt) => opt.clone(),
        Command::FindAndSendTags(opt) => opt.clone(),
    };

    let handler = HandlerBuilder::new()
        .api_key(&opt.saucenao_key)
        .min_similarity(80)
        .db(Handler::PIXIV)
        .build();

    let hydrus = Hydrus::new(Client::new(&args.hydrus_url, &args.hydrus_key));
    let pixiv = PixivClient::new();

    let tags = opt.tags.into_iter().map(Tag::from).collect();
    let service = ServiceName(opt.tag_service);

    let files = hydrus.search().add_tags(tags).run().await.unwrap();
    log::info!("Found {} files", files.len());
    let tmpdir = TempDir::new("hydrus-files").unwrap();

    let sleep_duration = Duration::from_secs(6);

    for mut file in files {
        let start = Instant::now();
        match &args.subcommand {
            Command::FindAndSendUrl(_) => {
                let _ = search_and_send_urls(&hydrus, &handler, &tmpdir, &mut file).await;
            }
            Command::FindAndSendTags(_) => {
                let _ = tag_file(
                    opt.finish_tag.as_ref(),
                    &handler,
                    &pixiv,
                    &service,
                    &tmpdir,
                    &mut file,
                )
                .await;
            }
        }
        let elapsed = start.elapsed();

        if elapsed.as_secs() < 8 {
            tokio::time::sleep(sleep_duration - elapsed).await; // rate limit of 6# / 30s
        }
    }
}

async fn tag_file(
    finish_tag: Option<&String>,
    handler: &Handler,
    pixiv: &PixivClient,
    service: &ServiceName,
    tmpdir: &TempDir,
    mut file: &mut HydrusFile,
) -> Result<()> {
    if let Err(e) = search_and_assign_tags(&handler, &pixiv, &service, &tmpdir, &mut file).await {
        let hash = file.hash().await.unwrap();
        log::error!("Failed to search tags to file {}: {:?}", hash, e);
    } else if let Some(finish_tag) = finish_tag {
        file.add_tags(service.clone().into(), vec![finish_tag.into()])
            .await
            .unwrap();
    }

    Ok(())
}

async fn search_and_send_urls(
    hydrus: &Hydrus,
    handler: &Handler,
    tmpdir: &TempDir,
    file: &mut HydrusFile,
) -> Result<()> {
    let sauces = get_sauces_for_file(&handler, tmpdir, file).await?;
    let urls = get_urls(&sauces);
    for url in urls {
        hydrus.import().url(url).run().await?;
    }

    Ok(())
}

async fn search_and_assign_tags(
    handler: &Handler,
    pixiv: &PixivClient,
    service: &ServiceName,
    tmpdir: &TempDir,
    mut file: &mut HydrusFile,
) -> Result<()> {
    log::debug!("Getting tags for hydrus file {:?}", file.id);
    let sauces = get_sauces_for_file(&handler, tmpdir, file).await?;

    assign_pixiv_tags_and_url(&pixiv, service, &mut file, &sauces).await
}

async fn get_sauces_for_file(
    handler: &Handler,
    tmpdir: &TempDir,
    mut file: &mut HydrusFile,
) -> Result<Vec<Sauce>> {
    log::debug!("Creating tmp file for hydrus file {:?}", file.id);
    let path = search::create_tmp_sauce_file(&tmpdir, &mut file).await?;
    log::debug!("Getting sauce for hydrus file {:?}", file.id);

    let sauce = handler.get_sauce(path.to_str().unwrap(), None, None)?;
    log::debug!("Getting tags for hydrus file {:?}", file.id);
    Ok(sauce)
}

async fn assign_pixiv_tags_and_url(
    pixiv: &&PixivClient,
    service: &ServiceName,
    file: &mut &mut HydrusFile,
    sauce: &Vec<Sauce>,
) -> Result<()> {
    let hash = file.hash().await?;
    if let Some(url) = search::get_pixiv_url(&sauce) {
        let tags = search::get_tags_for_sauce(&pixiv, url).await?;

        if tags.len() > 0 {
            log::info!("Found {} tags for file {:?}", tags.len(), hash);
            file.add_tags(service.clone().into(), tags).await?;
        } else {
            log::info!("No tags for file {:?} found", hash);
        }
        file.associate_urls(vec![url.to_string()]).await?;
    } else {
        log::info!("No pixiv post for file {:?} found", hash);
    }

    Ok(())
}
