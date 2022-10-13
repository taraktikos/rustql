use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use std::env;
use diesel::{Insertable, Queryable};
use chrono::NaiveDateTime;
use crate::diesel_schema::users;

pub type PostgresPool = Pool<ConnectionManager<PgConnection>>;

pub fn get_pool() -> PostgresPool {
    dotenvy::dotenv().unwrap();
    let url = env::var("DATABASE_URL").expect("no DB URL");
    let mgr = ConnectionManager::<PgConnection>::new(url);
    r2d2::Pool::builder()
        .build(mgr)
        .expect("could not build connection pool") // TODO: handle errors
}


#[derive(Queryable)]
#[diesel(table_name = users)]
pub struct DBUser {
    pub id: i64,
    pub email: String,
    pub password: Vec<u8>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct DBNewUser {
    pub email: String,
    pub password: Vec<u8>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}
