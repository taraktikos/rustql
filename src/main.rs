use std::io;
use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, middleware, Responder, route, web, get};
use actix_web::web::Data;
use actix_web_lab::respond::Html;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use crate::context::GraphQLContext;
use crate::db::{get_pool, PostgresPool};
use crate::schema::{create_schema, Schema};


mod schema;
mod db;
mod context;
mod diesel_schema;

#[route("/graphql", method = "GET", method = "POST")]
async fn graphql(
    pool: Data<PostgresPool>,
    st: Data<Schema>,
    data: web::Json<GraphQLRequest>,
) -> impl Responder {
    let ctx = GraphQLContext {
        pool: pool.get_ref().to_owned(),
    };

    let user = data.execute(&st, &ctx).await;
    HttpResponse::Ok().json(user)
}

#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html(graphiql_source("/graphql", None))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let schema = Arc::new(create_schema());

    log::info!("starting HTTP server on port 8080");
    log::info!("GraphiQL playground: http://localhost:8080/graphiql");

    let pool = get_pool();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::from(schema.clone()))
            .service(graphql)
            .service(graphql_playground)
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
    })
        .workers(2)
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
