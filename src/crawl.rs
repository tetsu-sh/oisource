use crate::article::{Article, QiitaArticle};
use crate::utils::errors::MyError;
use async_trait::async_trait;
use chrono::Local;
use reqwest;
use std::{collections::HashMap, env};

pub async fn crawl() -> Result<Vec<Article>, MyError> {
    // qiita
    let access_token = env::var("QIITA_ACCESS_TOKEN").expect("qiita access token is not set");
    let qiita_user_id = env::var("QIITA_USER_ID").expect("qiita user id is not set");
    let qiita_qrawler = QiitaCrawler::new(access_token, qiita_user_id);
    let qiita_articles = qiita_qrawler.fetch().await?;
    println!("{}", qiita_articles.len());
    Ok(qiita_articles)
}

#[async_trait]
trait Crawl {
    fn media(&self) -> String;
    async fn fetch(&self) -> Result<Vec<Article>, MyError>;
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
        let crawled_at = Local::now().naive_local().to_string();
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
    // stockをがなくなると空の配列が帰ってくる。
    async fn fetch(&self) -> Result<Vec<Article>, MyError> {
        let client = reqwest::Client::new();
        let mut page_num = 1;
        let mut qiita_articles: Vec<QiitaArticle> = vec![];
        loop {
            let body = client
                .get(format!(
                    "https://qiita.com/api/v2/users/{}/stocks?page={}",
                    self.user_id, page_num
                ))
                .bearer_auth(self.access_token.clone())
                .send()
                .await?
                .text()
                .await?;
            println!("{}", body.len());
            if body == "[]" {
                break;
            }
            let mut partial_qiita_articles: Vec<QiitaArticle> = serde_json::from_str(&body)?;
            println!("{}", partial_qiita_articles.len());
            qiita_articles.append(&mut partial_qiita_articles);
            page_num += 1;
        }
        println!("{}", qiita_articles.len());

        let articles = qiita_articles
            .iter()
            .map(|qiita_article| qiita_article.to_article(self.media(), self.crawled_at.clone()))
            .collect::<Vec<Article>>();
        Ok(articles)
    }
}
