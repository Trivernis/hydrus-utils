use lazy_regex::regex;

pub enum UrlType {
    Reddit,
    Twitter,
    Other,
}

pub fn find_url_type(url: &str) -> UrlType {
    if is_reddit_url(url) {
        UrlType::Reddit
    } else if is_twitter_url(url) {
        UrlType::Twitter
    } else {
        UrlType::Other
    }
}

fn is_reddit_url(url: &str) -> bool {
    let r = regex!(r#"^http(s)?://(www\.)?(reddit\.com|redd\.it).*$"#i);
    r.is_match(url)
}

fn is_twitter_url(url: &str) -> bool {
    let r = regex!(r#"^http(s)?://(www\.)?twitter\.com.*$"#);
    r.is_match(url)
}
