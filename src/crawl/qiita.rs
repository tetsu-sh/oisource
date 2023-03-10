use crate::article::{Article, DatetimeFormatter};
use crate::store;
use crate::store::model::store_rdb;
use crate::utils::errors::MyError;
use actix_web::HttpResponse;
use async_trait::async_trait;
use chrono::Local;
use reqwest::{self, Client};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::{collections::HashMap, env};

use super::Media;

#[derive(Serialize, Deserialize, Debug)]
pub struct QiitaArticle {
    id: String,
    title: String,
    url: String,
    created_at: String,
    user: QiitaUser,
}

impl QiitaArticle {
    pub fn to_article(&self, media: String, crawled_at: String) -> Article {
        Article {
            id: self.id.clone(),
            title: self.title.clone(),
            author: self.user.name.clone(),
            media,
            url: self.url.clone(),
            summary: "".to_string().clone(),
            created_at: DatetimeFormatter::qiita_to(&self.created_at),
            crawled_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QiitaUser {
    name: String,
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
        Media::Qiita.to_string()
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

pub async fn qiita_crawl() -> Result<Vec<Article>, MyError> {
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

pub async fn crawl_to_update(latest_one: Article) -> Result<Vec<Article>, MyError> {
    // qiita
    let access_token = env::var("QIITA_ACCESS_TOKEN").expect("qiita access token is not set");
    let qiita_user_id = env::var("QIITA_USER_ID").expect("qiita user id is not set");
    let qiita_qrawler = QiitaCrawler::new(access_token, qiita_user_id);
    let mut page_num = 1;
    let per_page = 20;
    // fetch items.
    // 20?????????????????????????????????latest??????????????????  ???????????????????????????O(n)???????????????????????????????????????OK
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
