// @generated automatically by Diesel CLI.

diesel::table! {
    articles (id) {
        id -> Varchar,
        title -> Varchar,
        auther -> Varchar,
        media -> Varchar,
        url -> Varchar,
        summary -> Varchar,
        created_at -> Datetime,
        crawled_at -> Datetime,
    }
}
