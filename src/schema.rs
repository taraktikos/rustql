use std::pin::Pin;
use diesel::prelude::*;
use diesel::deserialize::{QueryableByName};
use diesel::{RunQueryDsl, sql_query};
use juniper::{FieldError, FieldResult, futures, graphql_object, graphql_subscription, graphql_value, RootNode};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};
use crate::context::GraphQLContext;

table! {
    user (id) {
        id -> Int8,
    }
}

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[derive(GraphQLObject, Debug)]
#[graphql(description = "Human description")]
struct Human {
    pub id: i32,
}

#[derive(QueryableByName, Debug)]
#[diesel(table_name = user)]
struct DBHuman {
    pub id: i64,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "NewHuman description")]
struct NewHuman {
    id: i32,
}


pub struct QueryRoot;

#[graphql_object(context = GraphQLContext)]
impl QueryRoot {
    fn human(context: &GraphQLContext, _id: i32) -> FieldResult<Human> {
        let mut conn = context.pool.get()?;
        sql_query(r#"SELECT * FROM "user" ORDER BY id"#).load::<DBHuman>(&mut conn)?
            .first()
            .map(|dbh| Human { id: dbh.id as i32 })
            .ok_or(FieldError::new("not found", graphql_value!({"internal_error": "Database error"})))
    }
}

pub struct MutationRoot;

#[graphql_object(context = GraphQLContext)]
impl MutationRoot {
    fn create_human(new_human: NewHuman) -> FieldResult<Human> {
        Ok(Human {
            id: new_human.id,
        })
    }
}

pub struct Subscription;

type HumanStream = Pin<Box<dyn futures::Stream<Item=Result<Human, FieldError>> + Send>>;

#[graphql_subscription(context = GraphQLContext)]
impl Subscription {
    #[graphql(description = "Random human")]
    async fn random_human(_context: &GraphQLContext) -> HumanStream {
        let stream = async_stream::stream! {
            yield Ok(Human {
                id: 123,
            })
        };
        Box::pin(stream)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, Subscription>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, Subscription)
}
