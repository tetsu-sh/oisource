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

const YOUTUBE_API_BASE_URL: &str = "https://www.googleapis.com/youtube/";
const TWITTER_API_BASE_URL: &str = "https://api.twitter.com/2/";

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

pub async fn twitter_crawl() -> Result<Vec<Article>, MyError> {
    const media: &str = "twitter";
    let crawled_at = Local::now().naive_local().to_string();
    let twitter_user_id = env::var("TWITTER_USER_ID").expect("twitter user id is not set");
    let bearer_token = env::var("TWITTER_BEARER_TOKEN").expect("twitter bearer token is not set");

    let client = reqwest::Client::new();
    let mut next_page_token = "".to_string();
    let mut articles = vec![];
    loop {
        let favorite_res =
            fetch_twitter_favorite(&client, &twitter_user_id, &bearer_token, &next_page_token)
                .await?;
        // usersから該当のuserをauther_idで検索する
        let mut part_of_articles = favorite_res
            .data
            .into_iter()
            .map(|tweet| {
                tweet.to_article(
                    favorite_res
                        .includes
                        .users
                        .iter()
                        .find(|&user| &user.id == &tweet.auther_id)
                        .unwrap()
                        .username
                        .clone(),
                    media.to_owned(),
                    crawled_at.clone(),
                )
            })
            .collect::<Vec<Article>>();
        articles.append(&mut part_of_articles);
        match favorite_res.meta.next_token {
            Some(t) => next_page_token = t,
            None => break,
        }
    }
    Ok(articles)
}

async fn fetch_twitter_favorite(
    client: &Client,
    user_id: &str,
    bearer_token: &str,
    page_token: &str,
) -> Result<FavoriteRes, MyError> {
    let query_params = [
        ("pagenation_token", page_token.to_string()),
        ("expansions", "auther_id".to_string()),
        ("tweet.fields", "created_at".to_string()),
    ];
    let raw_res = client
        .get(format!(
            "{}users/{}/liked_tweets",
            TWITTER_API_BASE_URL, user_id
        ))
        .query(&query_params)
        .bearer_auth(bearer_token)
        .send()
        .await?
        .text()
        .await?;
    return Ok(serde_json::from_str::<FavoriteRes>(&raw_res)?);
}

/// twitter favorite api response schema.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct FavoriteRes {
    data: Vec<Tweet>,
    meta: TweetMeta,
    includes: Expansion,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Expansion {
    users: Vec<TweetUser>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TweetUser {
    id: String,
    name: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TweetMeta {
    result_count: isize,
    next_token: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Tweet {
    edit_history_tweet_ids: Vec<String>,
    id: String,
    auther_id: String,
    created_at: String,
    text: String,
}

impl Tweet {
    fn to_article(&self, auther: String, media: String, crawled_at: String) -> Article {
        Article {
            id: self.id.clone(),
            title: self.text.clone(),
            auther: auther.clone(),
            media,
            url: format!("https://twitter.com/{}/status/{}", auther, self.id),
            // summaryはないので、text.
            summary: self.text.clone(),
            created_at: DatetimeFormatter::twitter_to(&self.created_at),
            crawled_at,
        }
    }
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
        let mut playlistres = fetch_youtube_playlists(
            &client,
            &api_key,
            &channel_id,
            &next_page_token_for_playlists,
        )
        .await?;
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
            let playlistitemsres = fetch_youtube_items(
                &client,
                &api_key,
                &playlist.id,
                &next_page_token_for_playlistitems,
            )
            .await?;
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
/// if items exists then return next_page_token
async fn fetch_youtube_items(
    client: &Client,
    api_key: &str,
    playlist_id: &str,
    page_token: &str,
) -> Result<PlayListItemRes, MyError> {
    let raw_res = client
        .get(YOUTUBE_API_BASE_URL.to_owned() + "v3/playlistItems")
        .query(&[
            ("key", api_key),
            ("playlistId", playlist_id),
            ("part", "id"),
            ("part", "snippet"),
            ("part", "contentDetails"),
            ("maxResults", "50"),
            ("pageToken", page_token),
        ])
        .send()
        .await?
        .text()
        .await?;
    Ok(serde_json::from_str::<PlayListItemRes>(&raw_res)?)
}

async fn fetch_youtube_playlists(
    client: &Client,
    api_key: &str,
    channel_id: &str,
    page_token: &str,
) -> Result<PlayListRes, MyError> {
    let res = client
        .get(YOUTUBE_API_BASE_URL.to_owned() + "v3/playlists")
        .query(&[
            ("key", api_key),
            ("channelId", channel_id),
            ("part", "id"),
            ("part", "snippet"),
            ("pageToken", page_token),
        ])
        .send()
        .await?
        .text()
        .await?;
    let playlistres: PlayListRes = serde_json::from_str(&res)?;
    Ok(playlistres)
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
