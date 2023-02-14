use std::str::FromStr;

use async_trait::async_trait;
use serde_json::json;
use strum_macros::Display;

use crate::{article::Article, utils::errors::MyError};

pub mod qiita;
pub mod twitter;
pub mod youtube;

#[async_trait]
trait Crawl {
    fn media(&self) -> String;
    async fn fetch(&self) -> Result<Vec<Article>, MyError>;
}

#[derive(Debug, Clone, Display)]
enum Media {
    Qiita,
    Youtube,
    Twitter,
}

impl FromStr for Media {
    type Err = MyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = match s {
            "qiita" => Ok(Self::Qiita),
            "youtube" => Ok(Self::Youtube),

            "twitter" => Ok(Self::Twitter),
            _ => {
                return Err(MyError::BadRequest(json!({"error":"error"})));
            }
        };
        return m;
    }
}
