use crate::article::{Article, DatetimeFormatter};
use crate::store;
use crate::store::model::store_rdb;
use crate::utils::errors::MyError;
use actix_web::HttpResponse;
use async_trait::async_trait;
use chrono::Local;
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, env};

const TWITTER_API_BASE_URL: &str = "https://api.twitter.com/2/";

pub async fn twitter_crawl() -> Result<Vec<Article>, MyError> {
    const media: &str = "twitter";
    let crawled_at = Local::now().naive_local().to_string();
    let twitter_user_id = env::var("TWITTER_USER_ID").expect("twitter user id is not set");
    let bearer_token = env::var("TWITTER_BEARER_TOKEN").expect("twitter bearer token is not set");

    let client = reqwest::Client::new();
    let mut next_page_token: Option<String> = None;
    let mut articles = vec![];
    loop {
        let favorite_res =
            fetch_twitter_favorite(&client, &twitter_user_id, &bearer_token, next_page_token)
                .await?;
        match favorite_res.data {
            Some(data) => {
                // usersから該当のuserをauthor_idで検索する
                let mut part_of_articles = data
                    .into_iter()
                    .map(|tweet| {
                        tweet.to_article(
                            favorite_res
                                .includes
                                .as_ref()
                                .unwrap()
                                .users
                                .iter()
                                .find(|&user| user.id == tweet.author_id)
                                .unwrap()
                                .username
                                .clone(),
                            media.to_owned(),
                            crawled_at.clone(),
                        )
                    })
                    .collect::<Vec<Article>>();
                articles.append(&mut part_of_articles);
            }
            None => break,
        }
        match favorite_res.meta.next_token {
            Some(t) => next_page_token = Some(t),
            None => break,
        }
    }
    Ok(articles)
}

/// itemがなくてもnext_tokenが帰ってくる。
/// そのnext_tokenを渡して帰ってくるものにitemはない。
async fn fetch_twitter_favorite(
    client: &Client,
    user_id: &str,
    bearer_token: &str,
    page_token: Option<String>,
) -> Result<FavoriteRes, MyError> {
    // 空文字 pagination_tokenは怒られる.
    let mut query_params = vec![
        ("expansions", "author_id".to_string()),
        ("tweet.fields", "created_at".to_string()),
    ];
    if let Some(tk) = page_token {
        query_params.push(("pagination_token", tk))
    };

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
    println!("{}", raw_res);
    return Ok(serde_json::from_str::<FavoriteRes>(&raw_res)?);
}

/// twitter favorite api response schema.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct FavoriteRes {
    data: Option<Vec<Tweet>>,
    meta: TweetMeta,
    includes: Option<Expansion>,
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
    author_id: String,
    created_at: String,
    text: String,
}

impl Tweet {
    fn to_article(&self, author: String, media: String, crawled_at: String) -> Article {
        Article {
            id: self.id.clone(),
            title: self.text.clone(),
            author: author.clone(),
            media,
            url: format!("https://twitter.com/{}/status/{}", author, self.id),
            // summaryはないので、text.
            summary: self.text.clone(),
            created_at: DatetimeFormatter::twitter_to(&self.created_at),
            crawled_at,
        }
    }
}

struct TweetCrawler {
    access_token: String,
}
