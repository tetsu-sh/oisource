use crate::schema::articles;
use crate::utils::db::establish_connection;
use crate::utils::errors::MyError;
use async_graphql::SimpleObject;
use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use diesel::MysqlConnection;
use dotenv::dotenv;
use log::info;
use reqwest;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::{collections::HashMap, env};

#[derive(Serialize, Deserialize, Debug, SimpleObject, Clone)]
pub struct Article {
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

#[derive(Debug, Queryable, Insertable, Identifiable, Clone)]
#[table_name = "articles"]
struct ArticleRDB {
    pub id: String,
    pub title: String,
    pub auther: String,
    pub media: String,
    pub url: String,
    pub summary: String,
    pub created_at: NaiveDateTime,
    pub crawled_at: NaiveDateTime,
}

impl ArticleRDB {
    fn store(self, conn: &MysqlConnection) -> Result<(), MyError> {
        diesel::insert_into(articles::table)
            .values(&self)
            .execute(conn)?;
        Ok(())
    }
    fn store_batch(conn: &MysqlConnection, records: Vec<ArticleRDB>) -> Result<(), MyError> {
        diesel::insert_into(articles::table)
            .values(records)
            .execute(conn)?;
        Ok(())
    }

    fn scan(conn: &MysqlConnection) -> Result<Vec<Article>, MyError> {
        let articlerdbs = articles::table.load::<ArticleRDB>(conn)?;
        let articles = articlerdbs
            .into_iter()
            .map(|articlerdb| ArticleRDB::to_domain(articlerdb))
            .collect::<Vec<Article>>();
        Ok(articles)
    }

    fn from_domain(article: Article) -> ArticleRDB {
        info!("{:?}", article);
        ArticleRDB {
            id: article.id,
            title: article.title,
            auther: article.auther,
            media: article.media,
            url: article.url,
            summary: article.summary,
            created_at: NaiveDateTime::parse_from_str(&article.created_at, "%Y-%m-%dT%H:%M:%S%:z")
                .unwrap(),
            crawled_at: NaiveDateTime::parse_from_str(&article.crawled_at, "%Y-%m-%d %H:%M:%S%.9f")
                .unwrap(),
        }
    }

    fn to_domain(article_rdb: ArticleRDB) -> Article {
        Article {
            id: article_rdb.id,
            title: article_rdb.title,
            auther: article_rdb.auther,
            media: article_rdb.media,
            url: article_rdb.url,
            summary: article_rdb.summary,
            created_at: article_rdb.created_at.to_string(),
            crawled_at: article_rdb.crawled_at.to_string(),
        }
    }
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
            crawled_at: crawler.crawled_at.clone(),
        }
    }
}

pub async fn crawl() -> Result<(), MyError> {
    // qiita
    dotenv().ok();
    let conn = _establish_connection();
    let pool = establish_connection();
    let conn = pool.get()?;
    let access_token = env::var("QIITA_ACCESS_TOKEN").expect("qiita access token is not set");
    let qiita_user_id = env::var("QIITA_USER_ID").expect("qiita user id is not set");
    let qiita_qrawler = QiitaCrawler::new(access_token, qiita_user_id);
    let qiita_articles = qiita_qrawler.fetch().await?;
    let records = qiita_articles
        .into_iter()
        .map(|article| ArticleRDB::from_domain(article))
        .collect::<Vec<ArticleRDB>>();
    ArticleRDB::store_batch(&conn, records);

    Ok(())
}

pub async fn scan() -> Result<Vec<Article>, MyError> {
    dotenv().ok();
    let conn = _establish_connection();
    let pool = establish_connection();
    let conn = pool.get()?;
    let record = ArticleRDB::scan(&conn);
    record
}

fn _establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[derive(Serialize, Deserialize, Debug, SimpleObject, Clone)]
pub struct Response {
    articles: Vec<Article>,
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
            .map(|article| article.to_article(self))
            .collect::<Vec<Article>>();
        Ok(qiita_articles)
    }
}
