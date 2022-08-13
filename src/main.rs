mod args;
mod error;
mod operations;
pub mod utils;

use crate::error::Result;
use crate::operations::find_and_send_tags::find_and_send_tags;
use crate::operations::find_and_send_urls::find_and_send_urls;
use args::*;
use clap::Parser;
use hydrus_api::wrapper::service::ServiceName;
use hydrus_api::wrapper::tag::Tag;
use hydrus_api::{Client, Hydrus};
use pixiv_rs::PixivClient;
use rustnao::{Handler, HandlerBuilder};
use std::str::FromStr;
use tempdir::TempDir;
use tokio::time::{Duration, Instant};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init_logger();
    let args: Args = Args::parse();
    tracing::debug!("args: {args:?}");
    let hydrus = Hydrus::new(Client::new(&args.hydrus_url, &args.hydrus_key));

    match args.subcommand {
        Command::FindAndSendUrl(opt) => send_tags_or_urls(opt, hydrus, true).await,
        Command::FindAndSendTags(opt) => send_tags_or_urls(opt, hydrus, false).await,
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

async fn send_tags_or_urls(opt: Options, hydrus: Hydrus, send_urls: bool) -> Result<()> {
    let pixiv = PixivClient::new();

    let handler = HandlerBuilder::new()
        .api_key(&opt.saucenao_key)
        .min_similarity(80)
        .db(Handler::PIXIV)
        .build();

    let tags = opt.tags.into_iter().map(Tag::from).collect();
    let service = ServiceName(opt.tag_service);

    let files = hydrus.search().add_tags(tags).run().await.unwrap();
    tracing::info!("Found {} files", files.len());
    let tmpdir = TempDir::new("hydrus-files").unwrap();

    let sleep_duration = Duration::from_secs(6);

    for mut file in files {
        let start = Instant::now();
        if send_urls {
            let _ = find_and_send_urls(&hydrus, &handler, &tmpdir, &mut file).await;
        } else {
            let _ = find_and_send_tags(
                opt.finish_tag.as_ref(),
                &handler,
                &pixiv,
                &service,
                &tmpdir,
                &mut file,
            )
            .await;
        }
        let elapsed = start.elapsed();

        if elapsed.as_secs() < 8 {
            tokio::time::sleep(sleep_duration - elapsed).await; // rate limit of 6# / 30s
        }
    }

    Ok(())
}
