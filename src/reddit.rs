use reqwest::Client as HttpClient;
use serde::Deserialize;
use std::fmt;

const USER_AGENT: &str = "reqwest";

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub async fn reddit_access_token(
    http: &HttpClient,
    client_id: &str,
    client_secret: &str,
) -> Result<String, BoxError> {
    let form = reqwest::multipart::Form::new().text("grant_type", "client_credentials");
    let rsp = http
        .post("https://www.reddit.com/api/v1/access_token")
        .multipart(form)
        .basic_auth(client_id, Some(client_secret))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .json::<AccessTokenResponse>()
        .await?;

    Ok(rsp.access_token)
}

pub async fn hot_wholesome_meme(http: &HttpClient, access_token: &str) -> Result<Post, BoxError> {
    let rsp = http
        .get("https://oauth.reddit.com/r/wholesomememes/hot.json")
        .query(&[("limit", "2")]) // we want second hottest meme
        .bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .json::<Listing>()
        .await?;

    match rsp.data.children.last() {
        Some(child) => Ok(child.data.clone()),
        None => Err(Box::new(NoPosts)),
    }
}

#[derive(Debug, Clone)]
struct NoPosts;

impl std::error::Error for NoPosts {}

impl fmt::Display for NoPosts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no posts")
    }
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct Listing {
    data: ListingData,
}

#[derive(Deserialize)]
struct ListingData {
    children: Vec<Child>,
}

#[derive(Deserialize)]
struct Child {
    data: ChildData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChildData {
    pub title: String,
    pub permalink: String,
    pub url: Option<String>,
}

pub type Post = ChildData;
