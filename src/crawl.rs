use crate::article::{Article, DatetimeFormatter, Media, QiitaArticle};
use crate::store;
use crate::store::model::store_rdb;
use crate::utils::errors::MyError;
use actix_web::HttpResponse;
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

pub async fn youtube_crawl_unauthorized() -> Result<Vec<Article>, MyError> {
    let crawled_at = Local::now().naive_local().to_string();
    let api_key = env::var("YOUTUBE_API_KEY").expect("youtube api key is not set");
    let channel_id = env::var("YOUTUBE_CHANNEL_ID").expect("youtube channel id is not set");
    let client = reqwest::Client::new();
    let mut playlists = vec![];
    let mut next_page_token_for_playlists = "".to_string();
    // playlist一覧を取得
    // nextTokenがなくなるまで全取得
    loop {
        let res = client
            .get("https://www.googleapis.com/youtube/v3/playlists")
            .query(&[
                ("key", api_key.clone()),
                ("channelId", channel_id.clone()),
                ("part", "id".to_string()),
                ("part", "snippet".to_string()),
                ("pageToken", next_page_token_for_playlists),
            ])
            .send()
            .await?
            .text()
            .await?;
        let mut playlistres: PlayListRes = serde_json::from_str(&res)?;
        playlists.append(&mut playlistres.items);
        match playlistres.next_page_token {
            Some(t) => next_page_token_for_playlists = t,
            None => break,
        }
    }
    // 各playlist一覧からitemを取得
    // nextTokenがなくなるまで全取得
    let mut articles = vec![];
    let mut next_page_token_for_playlistitems = "".to_string();
    for playlist in playlists {
        loop {
            let res = client
                .get("https://www.googleapis.com/youtube/v3/playlistItems")
                .query(&[
                    ("key", api_key.clone()),
                    ("playlistId", playlist.id.clone()),
                    ("part", "id".to_string()),
                    ("part", "snippet".to_string()),
                    ("part", "contentDetails".to_string()),
                    ("maxResults", 50.to_string()),
                    ("pageToken", next_page_token_for_playlistitems.clone()),
                ])
                .send()
                .await?
                .text()
                .await?;
            let playlistitemsres: PlayListItemRes = serde_json::from_str(&res)?;
            let mut playlistitems = playlistitemsres
                .items
                .iter()
                .map(|playlistitem| {
                    playlistitem.to_article(
                        Media::Youtube.to_string(),
                        crawled_at.clone(),
                        playlist.snippet.title.clone(),
                    )
                })
                .collect::<Vec<Article>>();
            articles.append(&mut playlistitems);

            match playlistitemsres.next_page_token {
                Some(t) => next_page_token_for_playlistitems = t,
                None => break,
            }
        }
    }

    Ok(articles)
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
    snippet: PlayListSnippet,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct PlayListSnippet {
    title: String,
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
    content_details: ContentDetail,
}

impl PlayListItem {
    pub fn to_article(&self, media: String, crawled_at: String, playlist_name: String) -> Article {
        Article {
            id: self.id.clone(),
            title: self.snippet.title.clone(),
            // autherが取れない。
            auther: playlist_name,
            media,
            url: format!(
                "https://www.youtube.com/watch?v={}",
                self.content_details.video_id
            ),
            summary: self.snippet.description.clone(),
            // publiced_atはリストに入れられた日なので、コンテンツの作成日ではないが、やむをえず
            created_at: DatetimeFormatter::youtube_to(&self.snippet.published_at),
            crawled_at,
        }
    }
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

pub async fn youtube_crawl_authorized() -> Result<HttpResponse, MyError> {
    // korewo jissou
    // https://developers.google.com/youtube/v3/guides/auth/server-side-web-apps?hl=ja#httprest
    let oauth_client =
        env::var("YOUTUBE_OAUTH_CLIENT_ID").expect("youtube oauth client id not set");
    let client = reqwest::Client::new();
    let url = "https://accounts.google.com/o/oauth2/v2/auth";
    let state = "hogehoge".to_string();
    let params = [
        ("client_id", oauth_client),
        ("redirect_url", "http://localhost:8000".to_string()),
        ("response_type", "code".to_string()),
        (
            "scope",
            "https://www.googleapis.com/auth/youtube.readonly".to_string(),
        ),
        ("state", state),
        ("include_granted_scopes", true.to_string()), // ("access_type",),
                                                      // ("state",),
    ];
    let res = client.get(url).query(&params).send().await?.text().await?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(res))
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
