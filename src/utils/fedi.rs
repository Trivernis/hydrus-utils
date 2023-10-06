#![allow(unused)]
use std::collections::HashMap;

use crate::Result;
use lazy_regex::regex;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::ClientBuilder;
use reqwest::{redirect::Policy, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use std::fmt::Debug;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum EntryData {
    Page(PostData),
}

#[derive(Debug, Deserialize)]
struct PostData {
    id: String,
    name: String,
    attachment: Vec<Attachment>,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Attachment {
    Link { href: String },
}

pub async fn is_fedi_url(url: &str) -> bool {
    get_post(url).await.is_ok()
}

/// Returns all images associated with a post
#[tracing::instrument(level = "debug")]
pub async fn get_post_images<S: AsRef<str> + Debug>(post_url: S) -> Result<Vec<String>> {
    let post_data = get_post(post_url.as_ref()).await?;

    let urls = post_data
        .attachment
        .into_iter()
        .map(|p| {
            let Attachment::Link { href } = p;
            href
        })
        .collect();

    Ok(urls)
}

#[tracing::instrument(level = "debug")]
async fn get_post(url: &str) -> Result<PostData> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/activity+json"),
    );

    let client = ClientBuilder::default()
        .default_headers(headers)
        .user_agent(fakeit::user_agent::random_platform())
        .build()?;
    let mut response: EntryData = client.get(url).send().await?.json().await?;

    let EntryData::Page(post) = response;

    Ok(post)
}

#[tokio::test]
async fn it_retrieves_post_data() {
    let data = get_post("https://lemmy.blahaj.zone/post/113727")
        .await
        .unwrap();
    assert!(!data.attachment.is_empty());
}

#[tokio::test]
async fn it_retrieves_post_misskey() {
    let data = get_post("https://social.funkyfish.cool/notes/97ng0c9is3")
        .await
        .unwrap();
    assert!(!data.attachment.is_empty());
}

#[tokio::test]
async fn it_retrieves_post_images() {
    let images = get_post_images("https://lemmy.blahaj.zone/post/113727")
        .await
        .unwrap();
    assert!(!images.is_empty());
    assert!(images.get(0).unwrap().ends_with(".jpg"));
}
