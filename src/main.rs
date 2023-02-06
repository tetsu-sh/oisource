use actix_cors::Cors;
use actix_web::web::{get, post, Data};
use actix_web::{guard, http, middleware, web, App, HttpResponse, HttpServer, Result};
use async_graphql::EmptyMutation;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Object, Schema,
};
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use dotenv::dotenv;

#[macro_use]
extern crate diesel;
mod article;
mod constants;
mod crawl;
mod output;
mod schema;
mod store;
mod utils;
use article::Article;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use r2d2::PooledConnection;
use utils::errors::MyError;

struct QueryRoot;

fn get_conn() -> Result<PooledConnection<ConnectionManager<MysqlConnection>>, MyError> {
    let pool = utils::db::establish_connection();
    let conn = pool.get()?;
    Ok(conn)
}

#[Object]
impl QueryRoot {
    async fn scan(&self) -> Result<Vec<Article>, MyError> {
        let conn = get_conn()?;
        let res = store::model::scan(&conn)?;
        Ok(res)
    }
    async fn is_latest(&self, media: String) -> Result<bool, MyError> {
        let conn = get_conn()?;
        let stored_one = store::model::latest_one(&conn, &media)?;
        let crawled_one = crawl::latest_one().await?;
        Ok(stored_one == crawled_one)
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn qiita_crawl(&self) -> Result<Vec<Article>, MyError> {
        let res = crawl::qiita_crawl().await?;
        let conn = get_conn()?;
        store::model::store_rdb(&conn, &res);
        Ok(res)
    }
    /// 差分アップデート
    /// 追加のみ対応
    async fn crawl_and_store(&self, media: String) -> Result<Vec<Article>, MyError> {
        let conn = get_conn()?;
        let latest_one = store::model::latest_one(&conn, &media)?;
        let res = crawl::crawl_to_update(latest_one).await?;
        store::model::store_rdb(&conn, &res);
        Ok(res)
    }

    async fn youtube_crawl(&self) -> Result<Vec<Article>, MyError> {
        let res = crawl::youtube_crawl_unauthorized().await?;
        let conn = get_conn()?;
        store::model::store_rdb(&conn, &res);
        Ok(res)
    }

    // async fn youtube_crawl_with_auth(&self) -> Result<HttpResponse, MyError> {
    //     let res = crawl::youtube_crawl_authorized().await?;
    //     Ok(res)
    // }

    async fn twitter_crawl(&self) -> Result<Vec<Article>, MyError> {
        let res = crawl::twitter_crawl().await?;
        let conn = get_conn()?;
        store::model::store_rdb(&conn, &res);
        Ok(res)
    }

    async fn gen_json_from_store(&self) -> Result<Vec<Article>, MyError> {
        let conn = get_conn()?;
        let res = store::model::scan(&conn)?;
        output::write_json(&res);
        Ok(res)
    }
}

type OiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

async fn index(schema: Data<OiSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn index_playground() -> Result<HttpResponse> {
    let source = playground_source(GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source))
}

async fn authorize() -> Result<HttpResponse> {
    let res = crawl::youtube_crawl_authorized().await?;
    Ok(res)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let pool = utils::db::establish_connection();
    dotenv().ok();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:8000")
            .allowed_origin("http://localhost:8080")
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".rust-lang.org"))
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(api)
            .app_data(Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
            .service(web::resource("/").guard(guard::Post()).to(index))
    })
    .bind(("localhost", 8080))?
    .workers(1)
    .run()
    .await
}

pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
            .service(web::resource("/").guard(guard::Post()).to(index)), // .service(web::resource("/auth").guard(guard::Post()).to(authorize)),
    );
}
