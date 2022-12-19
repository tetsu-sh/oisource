// @generated automatically by Diesel CLI.

diesel::table! {
    articles (id) {
        id -> Varchar,
        title -> Nullable<Varchar>,
        auther -> Nullable<Varchar>,
        media -> Varchar,
        url -> Varchar,
        summary -> Nullable<Varchar>,
        created_at -> Datetime,
        crawled_at -> Datetime,
    }
}
