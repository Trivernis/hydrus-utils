use super::fedi;
use lazy_regex::regex;

pub enum UrlType {
    Reddit,
    Fedi,
    Other,
}

pub async fn find_url_type(url: &str) -> UrlType {
    if is_reddit_url(url) {
        UrlType::Reddit
    } else if fedi::is_fedi_url(url).await {
        UrlType::Fedi
    } else {
        UrlType::Other
    }
}

fn is_reddit_url(url: &str) -> bool {
    let r = regex!(r#"^http(s)?://(www\.)?(reddit\.com|redd\.it|reddit\.app\.link).*$"#i);
    r.is_match(url)
}
