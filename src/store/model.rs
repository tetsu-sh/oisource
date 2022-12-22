use crate::article::Article;
use crate::schema::articles;
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
    let record = ArticleRDB::scan(&conn);
    record
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
            .map(|articlerdb| ArticleRDB::to_domain(&articlerdb))
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

    fn to_domain(article_rdb: &ArticleRDB) -> Article {
        Article {
            id: article_rdb.id.clone(),
            title: article_rdb.title.clone(),
            auther: article_rdb.auther.clone(),
            media: article_rdb.media.clone(),
            url: article_rdb.url.clone(),
            summary: article_rdb.summary.clone(),
            created_at: article_rdb.created_at.to_string(),
            crawled_at: article_rdb.crawled_at.to_string(),
        }
    }
}
