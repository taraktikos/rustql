use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use std::env;

pub type PostgresPool = Pool<ConnectionManager<PgConnection>>;

pub fn get_pool() -> PostgresPool {
    dotenvy::dotenv().unwrap();
    let url = env::var("DATABASE_URL").expect("no DB URL");
    let mgr = ConnectionManager::<PgConnection>::new(url);
    r2d2::Pool::builder()
        .build(mgr)
        .expect("could not build connection pool") // TODO: handle errors
}
