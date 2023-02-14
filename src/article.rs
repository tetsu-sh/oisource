use std::{str::FromStr, string::ParseError};

use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Serialize, Deserialize, Debug, SimpleObject, Clone, PartialEq)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub author: String,
    pub media: String,
    pub url: String,
    pub summary: String,
    pub created_at: String,
    pub crawled_at: String,
}

pub struct DatetimeFormatter {}

impl DatetimeFormatter {
    pub fn qiita_to(datetime: &String) -> String {
        let naive_datetime =
            NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%:z").unwrap();
        naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    pub fn youtube_to(datetime: &String) -> String {
        let naive_datetime =
            NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%Z").unwrap();
        naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    pub fn twitter_to(datetime: &String) -> String {
        let naive_datetime =
            NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S.%Z").unwrap();
        naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
