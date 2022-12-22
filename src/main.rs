use actix_web::web::{get, post, Data};
use actix_web::{guard, middleware, web, App, HttpResponse, HttpServer, Result};
use async_graphql::EmptyMutation;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Object, Schema,
};
#[macro_use]
extern crate diesel;
mod article;
mod constants;
mod crawl;
mod fetch;
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
    async fn crawl(&self) -> Result<String, MyError> {
        let res = crawl::crawl().await?;
        Ok("sss".to_string())
    }

    async fn scan(&self) -> Result<Vec<Article>, MyError> {
        let res = fetch::fetch()?;
        Ok(res)
    }
}

type MuscleSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

async fn index(schema: Data<MuscleSchema>, req: GraphQLRequest) -> GraphQLResponse {
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
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let pool = utils::db::establish_connection();
    let app_state = utils::state::AppState { pool };

    HttpServer::new(move || {
        App::new()
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
