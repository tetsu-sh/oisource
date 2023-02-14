use crate::article::{Article, DatetimeFormatter};
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
                        "media".to_string(),
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
            // authorが取れない。
            author: playlist_name,
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
