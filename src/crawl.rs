use crate::article::{Article, Media, QiitaArticle};
use crate::store;
use crate::store::model::store_rdb;
use crate::utils::errors::MyError;
use async_trait::async_trait;
use chrono::Local;
use reqwest::{self, Client};
use serde_json::json;
use std::{collections::HashMap, env};

pub async fn crawl() -> Result<Vec<Article>, MyError> {
    // qiita
    let access_token = env::var("QIITA_ACCESS_TOKEN").expect("qiita access token is not set");
    let qiita_user_id = env::var("QIITA_USER_ID").expect("qiita user id is not set");
    let qiita_qrawler = QiitaCrawler::new(access_token, qiita_user_id);
    let mut articles = vec![];
    let mut page_num = 1;
    let per_page = 20;
    // fetch all items.
    loop {
        let mut partial_articles = qiita_qrawler.fetch(page_num, per_page).await?;
        if partial_articles.is_empty() {
            break;
        }
        articles.append(&mut partial_articles);
        page_num += 1;
    }
    Ok(articles)
}

pub async fn youtube_crawl_unauthorized() -> Result<(), MyError> {
    let api_key = env::var("YOUTUBE_API_KEY").expect("youtube api key is not set");
    let channel_id = env::var("YOUTUBE_CHANNEL_ID").expect("youtube channel id is not set");

    let client = reqwest::Client::new();
    let res = client
        .get("https://www.googleapis.com/youtube/v3/playlists")
        .query(&[
            ("key", api_key.clone()),
            ("channelId", channel_id.clone()),
            ("part", "id".to_string()),
            ("pageToken", "".to_string()),
        ])
        .send()
        .await?
        .text()
        .await?;

    println!("{:?}", res);
    let mut nest_page_token = "".to_string();
    let playlistres: PlayListRes = serde_json::from_str(&res)?;
    let playlist = playlistres.items;
    for playlist in playlist {
        let res = client
            .get("https://www.googleapis.com/youtube/v3/playlistItems")
            .query(&[
                ("key", api_key.clone()),
                ("playlistId", playlist.id),
                ("part", "id".to_string()),
                ("part", "snippet".to_string()),
            ])
            .send()
            .await?
            .text()
            .await?;
        println!("{:?}", res);
        let playlistitemsres: PlayListItemRes = serde_json::from_str(&res)?;

        println!("{:?}", playlistitemsres);
        let playlistitems = playlistitemsres.items;
        println!("{:?}", playlistitems);
    }

    Ok(())
}
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PlayListRes {
    next_page_token: Option<String>,
    items: Vec<PlayList>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct PlayList {
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PlayListItemRes {
    next_page_token: Option<String>,
    items: Vec<PlayListItem>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PlayListItem {
    id: String,
    snippet: PlayListItemSnippet,
    content_details: Option<ContentDetail>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PlayListItemSnippet {
    published_at: String,
    title: String,
    description: String,
    channel_title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ContentDetail {
    video_id: String,
}

pub fn youtube_crawl_authorized() -> Result<(), MyError> {
    let oauth_client =
        env::var("YOUTUBE_OAUTH_CLIENT_ID").expect("youtube oauth client id not set");
    let client = reqwest::Client::new();
    let url = "https://accounts.google.com/o/oauth2/v2/auth";
    let params = [
        ("client_id", oauth_client),
        ("redirect_url", "http://localhost:8000".to_string()),
        ("response_type", "code".to_string()),
        (
            "scope",
            "https://www.googleapis.com/auth/youtube.readonly".to_string(),
        ),
        // ("access_type",),
        // ("state",),
    ];
    // korewo jissou
    // https://developers.google.com/youtube/v3/guides/auth/server-side-web-apps?hl=ja#httprest
    Ok(())
}

pub async fn crawl_to_update(latest_one: Article) -> Result<Vec<Article>, MyError> {
    // qiita
    let access_token = env::var("QIITA_ACCESS_TOKEN").expect("qiita access token is not set");
    let qiita_user_id = env::var("QIITA_USER_ID").expect("qiita user id is not set");
    let qiita_qrawler = QiitaCrawler::new(access_token, qiita_user_id);
    let mut page_num = 1;
    let per_page = 20;
    // fetch items.
    // 20こくらいクロールして、latestと比較して、  一致するまで探す。O(n)だけど大した数じゃないのでOK
    let mut articles_to_update = vec![];
    loop {
        let partial_articles = qiita_qrawler.fetch(page_num, per_page).await?;
        for article in partial_articles.iter() {
            if latest_one == *article {
                break;
            } else {
                articles_to_update.push(article.clone());
            }
        }
        if partial_articles.is_empty() {
            break;
        }
        page_num += 1;
    }
    Ok(articles_to_update)
}

pub async fn latest_one() -> Result<Article, MyError> {
    // fetch latest item
    let access_token = env::var("QIITA_ACCESS_TOKEN").expect("qiita access token is not set");
    let qiita_user_id = env::var("QIITA_USER_ID").expect("qiita user id is not set");
    let qiita_qrawler = QiitaCrawler::new(access_token, qiita_user_id);
    let page_num = 1;
    let per_page = 1;
    let latest_one_by_crawl = qiita_qrawler
        .fetch(page_num, per_page)
        .await?
        .first()
        .expect("must be one")
        .clone();

    Ok(latest_one_by_crawl)
}

#[async_trait]
trait Crawl {
    fn media(&self) -> String;
    async fn fetch(&self) -> Result<Vec<Article>, MyError>;
}

struct TweetCrawler {
    access_token: String,
}

#[derive(Debug, Clone)]
struct QiitaCrawler {
    client: Client,
    access_token: String,
    user_id: String,
    crawled_at: String,
}

impl QiitaCrawler {
    fn new(access_token: String, user_id: String) -> Self {
        let crawled_at = Local::now().naive_local().to_string();
        let client = reqwest::Client::new();
        QiitaCrawler {
            client,
            access_token,
            user_id,
            crawled_at,
        }
    }
    fn media(&self) -> String {
        "qiita".to_string()
    }
    /// no item then return [].
    async fn fetch(&self, page_num: i32, per_page: i32) -> Result<Vec<Article>, MyError> {
        let body = self
            .client
            .get(format!(
                "https://qiita.com/api/v2/users/{}/stocks",
                self.user_id
            ))
            .query(&[("page", page_num), ("per_page", per_page)])
            .bearer_auth(self.access_token.clone())
            .send()
            .await?
            .text()
            .await?;

        let qiita_articles: Vec<QiitaArticle> = serde_json::from_str(&body)?;

        let articles = qiita_articles
            .iter()
            .map(|qiita_article| qiita_article.to_article(self.media(), self.crawled_at.clone()))
            .collect::<Vec<Article>>();
        Ok(articles)
    }
}
