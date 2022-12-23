use crate::article::Article;
use crate::schema::articles;
use crate::schema::articles::created_at;
use crate::schema::articles::id;
use crate::utils::errors::MyError;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use diesel::MysqlConnection;

pub fn store_rdb(conn: &MysqlConnection, records: &Vec<Article>) {
    let records = records
        .into_iter()
        .map(|article| ArticleRDB::from_domain(article))
        .collect::<Vec<ArticleRDB>>();
    ArticleRDB::store_batch(&conn, records);
}
pub fn scan(conn: &MysqlConnection) -> Result<Vec<Article>, MyError> {
    let records = ArticleRDB::scan(&conn);
    records
}

pub fn latest_one(conn: &MysqlConnection, media: String) -> Result<Article, MyError> {
    ArticleRDB::latest_one_in_media(conn, media)
}

#[derive(Debug, Queryable, Insertable, Identifiable, Clone)]
#[table_name = "articles"]
pub struct ArticleRDB {
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
    pub fn latest_one_in_media(conn: &MysqlConnection, media: String) -> Result<Article, MyError> {
        // そこまでのデータ数にはならないので、indexで対応する。
        // データが多くなりそうなら、latestテーブルなどを検討する。
        let record = articles::table
            .filter(articles::media.eq(media))
            .order_by(created_at.desc())
            .first::<ArticleRDB>(conn)?;
        Ok(record.to_domain())
    }

    fn scan(conn: &MysqlConnection) -> Result<Vec<Article>, MyError> {
        let articlerdbs = articles::table.load::<ArticleRDB>(conn)?;
        let articles = articlerdbs
            .into_iter()
            .map(|articlerdb| articlerdb.to_domain())
            .collect::<Vec<Article>>();
        Ok(articles)
    }

    fn from_domain(article: &Article) -> ArticleRDB {
        ArticleRDB {
            id: article.id.clone(),
            title: article.title.clone(),
            auther: article.auther.clone(),
            media: article.media.clone(),
            url: article.url.clone(),
            summary: article.summary.clone(),
            created_at: NaiveDateTime::parse_from_str(&article.created_at, "%Y-%m-%dT%H:%M:%S%:z")
                .unwrap(),
            crawled_at: NaiveDateTime::parse_from_str(&article.crawled_at, "%Y-%m-%d %H:%M:%S%.9f")
                .unwrap(),
        }
    }

    fn to_domain(&self) -> Article {
        Article {
            id: self.id.clone(),
            title: self.title.clone(),
            auther: self.auther.clone(),
            media: self.media.clone(),
            url: self.url.clone(),
            summary: self.summary.clone(),
            created_at: self.created_at.to_string(),
            crawled_at: self.crawled_at.to_string(),
        }
    }
}
