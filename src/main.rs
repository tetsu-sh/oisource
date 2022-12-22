use actix_cors::Cors;
use actix_web::web::{get, post, Data};
use actix_web::{guard, http, middleware, web, App, HttpResponse, HttpServer, Result};
use async_graphql::EmptyMutation;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Object, Schema,
};
use dotenv::dotenv;

#[macro_use]
extern crate diesel;
mod article;
mod constants;
mod crawl;
mod fetch;
mod output;
mod schema;
mod store;
mod utils;
use article::Article;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use log::info;
use utils::errors::MyError;

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn scan(&self) -> Result<Vec<Article>, MyError> {
        let pool = utils::db::establish_connection();
        let conn = pool.get()?;
        let res = store::model::scan(&conn)?;
        Ok(res)
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn crawl_and_store(&self) -> Result<Vec<Article>, MyError> {
        let res = crawl::crawl().await?;
        println!("{}", res.len());
        let pool = utils::db::establish_connection();
        let conn = pool.get()?;
        store::model::store_rdb(&conn, &res);
        Ok(res)
    }

    async fn gen_csv_from_record(&self) -> Result<Vec<Article>, MyError> {
        let pool = utils::db::establish_connection();
        let conn = pool.get()?;
        let res = fetch::fetch(&conn)?;
        output::write_csv(&res);
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

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let pool = utils::db::establish_connection();
    let app_state = utils::state::AppState { pool };
    dotenv().ok();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:8000")
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
    .bind(("127.0.0.1", 8000))?
    .workers(1)
    .run()
    .await
}

pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
            .service(web::resource("/").guard(guard::Get()).to(index)),
    );
}
