use std::io;
use std::sync::Arc;
use std::time::Duration;
use actix_cors::Cors;
use actix_web::{App,HttpRequest, HttpResponse, HttpServer, middleware, Responder, route, web, get};
use actix_web::Error;
use actix_web::web::Data;
use actix_web_lab::respond::Html;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use context::GraphQLContext;
use db::{get_pool, PostgresPool};
use schema::{create_schema, Schema};
use juniper_graphql_ws::ConnectionConfig;
use juniper_actix::{ subscriptions::subscriptions_handler};

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

async fn subscriptions(
    pool: Data<PostgresPool>,
    req: HttpRequest,
    stream: web::Payload,
    schema: web::Data<Schema>,
) -> Result<HttpResponse, Error> {
    let ctx = GraphQLContext {
        pool: pool.get_ref().to_owned(),
    };
    let schema = schema.into_inner();
    let config = ConnectionConfig::new(ctx);
    // set the keep alive interval to 15 secs so that it doesn't timeout in playground
    // playground has a hard-coded timeout set to 20 secs
    let config = config.with_keep_alive_interval(Duration::from_secs(15));

    subscriptions_handler(req, stream, schema, config).await
}

#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html(graphiql_source("/graphql", Some("/subscriptions")))
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
            .service(web::resource("/subscriptions").route(web::get().to(subscriptions)))
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
    })
        .workers(2)
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
