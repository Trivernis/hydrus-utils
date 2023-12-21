mod args;
mod config;
mod error;
mod operations;
pub mod utils;

use crate::config::Config;
use crate::config::SauceNaoConfig;
use crate::error::Result;
use crate::operations::find_and_send_fedi_posts::find_and_send_fedi_posts;
use crate::operations::find_and_send_tags::find_and_send_tags;
use crate::operations::find_and_send_urls::find_and_send_urls;
use args::*;
use clap::Parser;
use hydrus_api::api_core::common::FileIdentifier;
use hydrus_api::api_core::endpoints::adding_tags::TagAction;
use hydrus_api::wrapper::service::ServiceName;
use hydrus_api::wrapper::tag::Tag;
use hydrus_api::{Client, Hydrus};
use operations::find_and_send_reddit_posts::find_and_send_reddit_posts;
use pixiv_rs::PixivClient;
use rustnao::{Handler, HandlerBuilder};
use std::str::FromStr;
use tempdir::TempDir;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{Duration, Instant};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use utils::urls::find_url_type;
use utils::urls::UrlType;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    color_eyre::install().unwrap();
    init_logger();
    let args: Args = Args::parse();
    let config = Config::read().expect("Failed to read configuration");
    tracing::debug!("args: {args:?}");
    let hydrus = Hydrus::new(Client::new(&config.hydrus.api_url, &config.hydrus.api_key));

    match args.subcommand {
        Command::FindAndSendUrl(opt) => {
            send_tags_or_urls(opt, config.into_saucenao(), hydrus, true).await
        }
        Command::FindAndSendTags(opt) => {
            send_tags_or_urls(opt, config.into_saucenao(), hydrus, false).await
        }
        Command::ImportRedditPosts(opt) => import_reddit_posts(opt, hydrus).await,
        Command::ImportFediPosts(opt) => import_fedi_posts(opt, hydrus).await,
        Command::ImportUrls(opt) => import_urls(opt, hydrus).await,
        Command::Tag(opt) => tag_files(opt, hydrus).await,
    }
    .expect("Failed to send tags or urls");
}

fn init_logger() {
    const DEFAULT_ENV_FILTER: &str = "info";
    let filter_string =
        std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_ENV_FILTER.to_string());
    let env_filter =
        EnvFilter::from_str(&*filter_string).expect("failed to parse env filter string");
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_env_filter(env_filter)
        .with_writer(std::io::stdout)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .compact()
        .init();
}

#[tracing::instrument(level = "debug", skip(hydrus))]
async fn send_tags_or_urls(
    opt: LookupOptions,
    saucenao_cfg: SauceNaoConfig,
    hydrus: Hydrus,
    send_urls: bool,
) -> Result<()> {
    let pixiv = PixivClient::new();

    let handler = HandlerBuilder::new()
        .api_key(&saucenao_cfg.api_key)
        .min_similarity(80)
        .db(Handler::PIXIV)
        .build();

    let tags = opt.tags.into_iter().map(Tag::from).collect();
    let service = ServiceName(opt.tag_service);

    let files = hydrus.search().add_tags(tags).run().await.unwrap();
    tracing::info!("Found {} files", files.len());
    let tmpdir = TempDir::new("hydrus-files").unwrap();

    let sleep_duration = Duration::from_secs(6);
    let total_files = files.len();
    let service_key = hydrus.get_service_key(service.into()).await?;

    for (i, mut file) in files.into_iter().enumerate() {
        let start = Instant::now();
        tracing::info!("Searching for file {} out of {}", i + 1, total_files);

        if send_urls {
            let _ = find_and_send_urls(&hydrus, &handler, &tmpdir, &mut file).await;
        } else {
            let _ = find_and_send_tags(
                opt.finish_tag.as_ref(),
                &handler,
                &pixiv,
                &service_key,
                &tmpdir,
                &mut file,
            )
            .await;
        }
        let elapsed = start.elapsed();

        if elapsed.as_secs() < 8 && sleep_duration > elapsed {
            tokio::time::sleep(sleep_duration - elapsed).await; // rate limit of 6# / 30s
        }
    }

    Ok(())
}

#[tracing::instrument(level = "debug", skip(hydrus))]
async fn import_reddit_posts(opt: ImportUrlsOptions, hydrus: Hydrus) -> Result<()> {
    let urls = get_urls_from_args(opt).await?;
    find_and_send_reddit_posts(&hydrus, urls).await
}

#[tracing::instrument(level = "debug", skip(hydrus))]
async fn import_fedi_posts(opt: ImportUrlsOptions, hydrus: Hydrus) -> Result<()> {
    let urls = get_urls_from_args(opt).await?;
    find_and_send_fedi_posts(&hydrus, urls).await
}

async fn import_urls(opt: ImportUrlsOptions, hydrus: Hydrus) -> Result<()> {
    let urls = get_urls_from_args(opt).await?;
    let mut reddit_urls = Vec::new();
    let mut fedi_urls = Vec::new();
    let mut unknown_urls = Vec::new();

    for url in urls {
        match find_url_type(&url).await {
            UrlType::Reddit => reddit_urls.push(url),
            UrlType::Fedi => fedi_urls.push(url),
            UrlType::Other => {
                tracing::warn!("Unknown url type {url}");
                unknown_urls.push(url)
            }
        }
    }
    tracing::info!("Importing reddit posts...");
    find_and_send_reddit_posts(&hydrus, reddit_urls).await?;
    find_and_send_fedi_posts(&hydrus, fedi_urls).await?;

    tracing::info!("Importing unknown urls...");

    for url in unknown_urls {
        if let Err(e) = hydrus.import().url(&url).run().await {
            tracing::error!("Failed to import {url}: {e}")
        }
    }

    Ok(())
}

async fn get_urls_from_args(opt: ImportUrlsOptions) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    if let Some(input_file) = opt.input {
        let file = File::open(input_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.len() > 0 {
                urls.push(line);
            }
        }
    } else if let Some(args_urls) = opt.urls {
        urls = args_urls;
    } else {
        panic!("No urls provided");
    }
    Ok(urls)
}

async fn tag_files(opt: TagOptions, hydrus: Hydrus) -> Result<()> {
    let tags = opt.tags.into_iter().map(Tag::from).collect::<Vec<_>>();
    let service_key = hydrus
        .get_service_key(ServiceName(opt.tag_service).into())
        .await?;

    for file in opt.files {
        hydrus
            .file(FileIdentifier::hash(file))
            .await?
            .add_tags(service_key.to_owned(), tags.clone())
            .await?;
    }
    Ok(())
}
