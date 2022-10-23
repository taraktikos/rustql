use std::{env, io};
use std::io::{stdout, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use actix_cors::Cors;
use actix_web::{App, get, HttpRequest, HttpResponse, HttpServer, middleware, Responder, route, web};
use actix_web::Error;
use actix_web::web::Data;
use actix_web_lab::respond::Html;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use context::GraphQLContext;
use db::{get_pool, PostgresPool};
use schema::{create_schema, Schema};
use juniper_graphql_ws::ConnectionConfig;
use juniper_actix::subscriptions::subscriptions_handler;
use clap::{Parser, Subcommand};
use temporal_sdk_core::{ClientOptions, ClientOptionsBuilder, init_worker, Logger, MetricsExporter, OtelCollectorOptions, telemetry_init, TelemetryOptions, TelemetryOptionsBuilder, TraceExporter, Url, WorkerConfigBuilder};

mod schema;
mod db;
mod context;
mod diesel_schema;
mod mailer;

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

#[derive(Subcommand, Debug)]
enum Action {
    Generate,
    Serve,
    SendEmail,
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Action,
}


fn get_server_options() -> ClientOptions {
    let temporal_server_address = "http://localhost:7233".to_owned();
    let url = Url::try_from(&*temporal_server_address).unwrap();
    ClientOptionsBuilder::default()
        .identity("integ_tester".to_string())
        .target_url(url)
        .client_name("temporal-core".to_string())
        .client_version("0.1.0".to_string())
        .build()
        .unwrap()
}

pub fn get_integ_telem_options() -> TelemetryOptions {
    let mut ob = TelemetryOptionsBuilder::default();
    if let Some(url) = env::var("OTEL_URL_ENV_VAR")
        .ok()
        .map(|x| x.parse::<Url>().unwrap())
    {
        let opts = OtelCollectorOptions {
            url,
            headers: Default::default(),
        };
        ob.tracing(TraceExporter::Otel(opts.clone()));
        ob.metrics(MetricsExporter::Otel(opts));
    }
    if let Some(addr) = env::var("PROM_ENABLE_ENV_VAR")
        .ok()
        .map(|x| SocketAddr::new([127, 0, 0, 1].into(), x.parse().unwrap()))
    {
        ob.metrics(MetricsExporter::Prometheus(addr));
    }
    ob.tracing_filter(env::var("RUST_LOG").unwrap_or_else(|_| "temporal_sdk_core=INFO".to_string()))
        .logging(Logger::Console)
        .build()
        .unwrap()
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    telemetry_init(&get_integ_telem_options()).expect("Telemetry inits cleanly");
    let worker_cfg = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("task_queue")
        .worker_build_id("worker_build_id")
        .build()
        .expect("Configuration options construct properly");

    let client = Arc::new(
        get_server_options()
            .connect(worker_cfg.namespace.clone(), None, None)
            .await
            .expect("Must connect"),
    );

    let _worker = init_worker(worker_cfg, client);

    // client.start_workflow()

    let args = Args::parse();

    match args.action {
        Action::Generate => generate().await,
        Action::Serve => serve().await,
        Action::SendEmail => mailer::send_email().await,
    }
}

async fn generate() -> io::Result<()> {
    let schema = create_schema();
    let result = schema.as_schema_language();

    stdout()
        .write(result.as_bytes())
        .map(|_| ())
}

async fn serve() -> io::Result<()> {
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
