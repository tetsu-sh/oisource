use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Serialize, Deserialize, Debug, SimpleObject, Clone, PartialEq)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub auther: String,
    pub media: String,
    pub url: String,
    pub summary: String,
    pub created_at: String,
    pub crawled_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QiitaArticle {
    id: String,
    title: String,
    url: String,
    created_at: String,
}

impl QiitaArticle {
    pub fn to_article(&self, media: String, crawled_at: String) -> Article {
        Article {
            id: self.id.clone(),
            title: self.title.clone(),
            auther: "".to_string(),
            media,
            url: self.url.clone(),
            summary: "".to_string().clone(),
            created_at: self.created_at.clone(),
            crawled_at,
        }
    }
}

#[derive(Debug, Clone, EnumString, Display)]
pub enum Media {
    Qiita,
}
