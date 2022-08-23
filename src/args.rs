use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Command,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Looks up files on saucenao and sends urls to hydrus to be imported
    #[clap(name = "send-url")]
    FindAndSendUrl(LookupOptions),

    /// Looks up files on saucenao and maps the tags found on pixiv to the files
    #[clap(name = "send-tags")]
    FindAndSendTags(LookupOptions),

    /// Looks up and imports reddit posts
    #[clap(name = "import-reddit-posts")]
    ImportRedditPosts(ImportRedditOptions),
}

#[derive(Parser, Debug, Clone)]
pub struct LookupOptions {
    /// The tag service the tags will be assigned to
    #[clap(long, default_value = "my tags")]
    pub tag_service: String,

    /// Tag that is assigned to files that have been processed
    #[clap(long)]
    pub finish_tag: Option<String>,

    /// Tags used to search for files
    #[clap(short, long)]
    pub tags: Vec<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct ImportRedditOptions {
    /// A file containing all urls with each
    /// url in a separate line
    #[clap(short, long)]
    pub input: Option<PathBuf>,

    /// A list of urls to import
    #[clap(short, long)]
    pub urls: Option<Vec<String>>,
}
