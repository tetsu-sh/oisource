use crate::article::Article;
use std::{fs::File, io::Write};

pub fn write_json(records: &Vec<Article>) {
    let s = serde_json::to_string(records).unwrap();
    let mut file = File::create("./source/source.json").unwrap();
    file.write_all(s.as_bytes()).unwrap();
}
