use anyhow::{Context, Result};
use async_graphql::SimpleObject;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use dotenv::dotenv;
use log::info;
use reqwest;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{collections::HashMap, env};

enum MyError {
    SerdeJson(serde_json::Error),
    Reqwest(reqwest::Error),
}

#[derive(Serialize, Deserialize, Debug, SimpleObject, Clone)]
struct Article {
    id: String,
    title: String,
    auther: String,
    media: String,
    url: String,
    summary: String,
    created_at: String,
    crawled_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct QiitaArticle {
    id: String,
    title: String,
    url: String,
    created_at: String,
}

impl QiitaArticle {
    fn to_article(&self, crawler: &QiitaCrawler) -> Article {
        Article {
            id: self.id.clone(),
            title: self.title.clone(),
            auther: "".to_string(),
            media: crawler.media().clone(),
            url: self.url.clone(),
            summary: "".to_string().clone(),
            created_at: self.created_at.clone(),
            crawled_at: crawler.crawled_at,
        }
    }
}

pub async fn all_fetch() -> Result<Response> {
    // qiita
    dotenv().ok();
    let access_token = env::var("QIITA_ACCESS_TOKEN").expect("qiita access token is not set");
    let qiita_user_id = env::var("QIITA_USER_ID").expect("qiita user id is not set");
    let qiita_qrawler = QiitaCrawler::new(access_token, qiita_user_id);
    let message = qiita_qrawler.fetch().await?;
    Ok(message)
}

#[derive(Serialize, Deserialize, Debug, SimpleObject, Clone)]
pub struct Response {
    articles: Vec<Article>,
}

#[async_trait]
trait Crawl {
    fn media(&self) -> String;
    async fn fetch(&self) -> Result<Response>;
}

struct TweetCrawler {
    access_token: String,
}

#[derive(Debug, Clone, PartialEq)]
struct QiitaCrawler {
    access_token: String,
    user_id: String,
    crawled_at: String,
}
impl QiitaCrawler {
    fn new(access_token: String, user_id: String) -> Self {
        let crawled_at = Local::now().to_string();
        QiitaCrawler {
            access_token,
            user_id,
            crawled_at,
        }
    }
}

#[async_trait]
impl Crawl for QiitaCrawler {
    fn media(&self) -> String {
        "qiita".to_string()
    }
    async fn fetch(&self) -> Result<Response> {
        let client = reqwest::Client::new();
        let mut map: HashMap<String, String> = HashMap::new();
        let body = client
            .get(format!(
                "https://qiita.com/api/v2/users/{}/stocks",
                self.user_id
            ))
            .bearer_auth(self.access_token.clone())
            .send()
            .await?
            .text()
            .await?;
        let articles: Vec<QiitaArticle> = serde_json::from_str(&body)?;
        let qiita_articles = articles
            .iter()
            .map(|article| article.to_article(self))
            .collect::<Vec<Article>>();
        Ok(Response {
            articles: qiita_articles,
        })
    }
}
