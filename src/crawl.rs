use crate::article::{Article, QiitaArticle};
use crate::store::model::store_rdb;
use crate::utils::db::establish_connection;
use crate::utils::errors::MyError;
use async_trait::async_trait;
use chrono::Local;
use dotenv::dotenv;
use reqwest;
use std::{collections::HashMap, env};

pub async fn crawl() -> Result<(), MyError> {
    // qiita
    dotenv().ok();
    let pool = establish_connection();
    let conn = pool.get()?;
    let access_token = env::var("QIITA_ACCESS_TOKEN").expect("qiita access token is not set");
    let qiita_user_id = env::var("QIITA_USER_ID").expect("qiita user id is not set");
    let qiita_qrawler = QiitaCrawler::new(access_token, qiita_user_id);
    let qiita_articles = qiita_qrawler.fetch().await?;
    write_csv(&qiita_articles);
    store_rdb(&conn, &qiita_articles);
    Ok(())
}

use csv::Writer;
pub fn write_csv(records: &Vec<Article>) {
    let mut wtr = Writer::from_path("test.csv").unwrap();
    for record in records.iter() {
        wtr.serialize(record);
    }
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
    async fn fetch(&self) -> Result<Vec<Article>, MyError> {
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
            .map(|article| article.to_article(self.media(), self.crawled_at.clone()))
            .collect::<Vec<Article>>();
        Ok(qiita_articles)
    }
}
