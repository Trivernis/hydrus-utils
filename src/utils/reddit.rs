#![allow(unused)]
use std::collections::HashMap;

use crate::Result;
use lazy_regex::regex;
use reqwest::ClientBuilder;
use reqwest::{redirect::Policy, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use std::fmt::Debug;

#[derive(Deserialize)]
#[serde(tag = "kind", content = "data")]
enum DataEntry {
    Listing(ListingEntry),
}

#[derive(Deserialize)]
struct ListingEntry {
    children: Vec<DataEntryChild>,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", content = "data")]
enum DataEntryChild {
    #[serde(alias = "t3")]
    T3(T3Data),
    #[serde(alias = "t1")]
    T1(HashMap<String, Value>),
    #[serde(alias = "more")]
    More(HashMap<String, Value>),
}

#[derive(Deserialize, Debug)]
struct T3Data {
    id: String,
    url: Option<String>,
    gallery_data: Option<GalleryData>,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
struct GalleryData {
    items: Vec<GalleryItem>,
}

#[derive(Deserialize, Debug)]
struct GalleryItem {
    media_id: String,
    id: u64,
}

/// Returns all images associated with a post
#[tracing::instrument(level = "debug")]
pub async fn get_post_images<S: AsRef<str> + Debug>(post_url: S) -> Result<Vec<String>> {
    let post_data = get_post(post_url.as_ref()).await?;

    if let Some(gallery_data) = post_data.gallery_data {
        let urls = gallery_data
            .items
            .into_iter()
            .map(|item| item.media_id)
            .map(|media_id| format!("https://i.redd.it/{}.jpg", media_id))
            .collect();
        Ok(urls)
    } else if let Some(url) = post_data.url {
        Ok(vec![url])
    } else {
        Ok(Vec::new())
    }
}

#[tracing::instrument(level = "debug")]
async fn get_post(url: &str) -> Result<T3Data> {
    let mut url = resolve_redirects(url).await?;

    // url cleanup
    // add trailing slash and remove path params
    if let Some((left, right)) = url.rsplit_once('?') {
        url = left.to_string();
    }
    if !url.ends_with('/') {
        url.push('/');
    }
    let client = ClientBuilder::default()
        .user_agent(fakeit::user_agent::random_platform())
        .build()?;
    let mut response: Vec<DataEntry> = client
        .get(format!("{}.json", url))
        .send()
        .await?
        .json()
        .await?;
    response.reverse();
    let first_entry = response.pop().unwrap();
    let mut first_listing = match first_entry {
        DataEntry::Listing(l) => l.children,
    };
    first_listing.reverse();
    let entry = first_listing.pop().unwrap();

    match entry {
        DataEntryChild::T3(t3) => Ok(t3),
        DataEntryChild::T1(_) | DataEntryChild::More(_) => panic!("Invalid data entry t1 or more"),
    }
}

/// Resolves reddit redirects
#[tracing::instrument(level = "debug")]
async fn resolve_redirects(url: &str) -> Result<String> {
    let mut url = url.to_string();

    for _ in 0..10 {
        if is_resolved(&url) {
            tracing::debug!("Url already resolved.");
            return Ok(url);
        }
        let client = reqwest::Client::builder()
            .user_agent(fakeit::user_agent::random_platform())
            .redirect(Policy::none())
            .build()?;
        let response = client.get(url).send().await?;

        if let Some(location) = response.headers().get("location") {
            tracing::debug!("Redirect to {location:?} found");
            url = location.to_str().unwrap().to_string();
        } else {
            tracing::debug!("No redirect found.");
            return Ok(response.url().as_str().to_string());
        }
    }

    Ok(url)
}

/// Checks if the url is already in a format that can be used for retrieving information
/// about the post
fn is_resolved(url: &str) -> bool {
    let r = regex!(r#"^http(s)?://(www\.)?reddit\.com/r/\S+?/comments/\S+$"#);
    r.is_match(url)
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn it_finds_post_images() {
        let images =
            super::get_post_images("https://www.reddit.com/r/196/comments/wmx2k3/dame_da_rule/")
                .await
                .unwrap();
        assert!(images.is_empty() == false);
    }

    #[tokio::test]
    async fn it_finds_post_images2() {
        let images = super::get_post_images("https://reddit.com/r/HentaiBullying/s/S1gKoG4s2S/")
            .await
            .unwrap();
        assert!(images.is_empty() == false);
    }

    #[tokio::test]
    async fn it_finds_multiple_post_images() {
        let images =
            super::get_post_images("https://www.reddit.com/r/dogelore/comments/wmas8c/le_yakuza/")
                .await
                .unwrap();
        assert!(images.is_empty() == false);
    }

    #[tokio::test]
    async fn it_finds_info_for_posts() {
        let post = super::get_post("https://www.reddit.com/r/196/comments/wmx2k3/dame_da_rule/")
            .await
            .unwrap();
        println!("{:?}", post.url);
        assert!(post.url.is_some());
    }
    #[tokio::test]
    async fn it_finds_info_for_gallery_posts() {
        let post = super::get_post("https://www.reddit.com/r/dogelore/comments/wmas8c/le_yakuza/")
            .await
            .unwrap();
        println!("{:?}", post.gallery_data);
        assert!(post.gallery_data.is_some());
        let gallery_data = post.gallery_data.unwrap();
        assert!(gallery_data.items.is_empty() == false)
    }
}
