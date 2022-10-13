use std::pin::Pin;
use chrono::{Utc};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use juniper::{FieldError, FieldResult, futures, graphql_object, graphql_subscription, RootNode};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};
use crate::context::GraphQLContext;
use crate::diesel_schema::users;
use crate::db::{DBNewUser, DBUser};


#[derive(GraphQLEnum)]
enum Roles {
    Admin,
    Viewer,
}

#[derive(GraphQLObject, Debug)]
#[graphql(description = "Human description")]
struct User {
    id: i32,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "NewHuman description")]
struct NewUser {
    email: String,
    password: String,
    first_name: Option<String>,
    last_name: Option<String>,
}

pub struct QueryRoot;

#[graphql_object(context = GraphQLContext)]
impl QueryRoot {
    fn user(context: &GraphQLContext, id: i32) -> FieldResult<User> {
        let mut conn = context.pool.get()?;

        let user = users::table
            .find(id as i64)
            .first::<DBUser>(&mut conn)?;

        Ok(User {
            id: user.id as i32,
            email: user.email.to_string(),
            first_name: user.first_name,
            last_name: user.last_name,
        })
    }

    fn users(context: &GraphQLContext) -> FieldResult<Vec<User>> {
        let mut conn = context.pool.get()?;

        let users = users::table
            .load::<DBUser>(&mut conn)?;

        let users_resp = users.iter().map(|user| User {
            id: user.id as i32,
            email: user.email.to_string(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
        }).collect();

        Ok(users_resp)
    }
}

pub struct MutationRoot;

#[graphql_object(context = GraphQLContext)]
impl MutationRoot {
    fn create_user(context: &GraphQLContext, new_user: NewUser) -> FieldResult<User> {
        let new_user = DBNewUser {
            email: new_user.email,
            password: new_user.password.as_bytes().to_vec(),
            first_name: new_user.first_name,
            last_name: new_user.last_name,
            created_at: Utc::now().naive_utc(),
            deleted_at: None,
        };

        let mut conn = context.pool.get()?;

        let id = diesel::insert_into(users::table)
            .values(&new_user)
            .returning(users::id)
            .get_result::<i64>(&mut conn)?;

        let user = users::table
            .find(id)
            .first::<DBUser>(&mut conn)?;

        Ok(User {
            id: user.id as i32,
            email: user.email.to_string(),
            first_name: user.first_name,
            last_name: user.last_name,
        })
    }
}

pub struct Subscription;

type HumanStream = Pin<Box<dyn futures::Stream<Item=Result<User, FieldError>> + Send>>;

#[graphql_subscription(context = GraphQLContext)]
impl Subscription {
    #[graphql(description = "Random human")]
    async fn random_human(_context: &GraphQLContext) -> HumanStream {
        let stream = async_stream::stream! {
            yield Ok(User {
                id: 123,
                email: "".to_string(),
                first_name: None,
                last_name: None,
            })
        };
        Box::pin(stream)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, Subscription>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, Subscription)
}
