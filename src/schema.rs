use std::pin::Pin;
use diesel::prelude::*;
use diesel::deserialize::{QueryableByName};
use diesel::{RunQueryDsl, sql_query};
use juniper::{FieldError, FieldResult, futures, graphql_object, graphql_subscription, RootNode};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};
use crate::context::GraphQLContext;

diesel::table! {
    user (id) {
        id -> Int8,
        email -> Nullable<Varchar>,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        password -> Nullable<Bytea>,
        created_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[table_name = "user"]
#[derive(GraphQLObject)]
#[graphql(description = "Human description")]
struct Human {
    id: String,
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "NewHuman description")]
struct NewHuman {
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}


pub struct QueryRoot;

#[graphql_object(context = GraphQLContext)]
impl QueryRoot {
    fn human(context: &GraphQLContext, _id: String) -> FieldResult<Human> {
        let mut conn = context.pool.get().unwrap();
        let users = sql_query("SELECT * FROM users ORDER BY id").load::<Human>(&mut conn)?;
        //    let expected_users = vec![
        //     User { id: 1, name: "Sean".into() },
        //     User { id: 2, name: "Tess".into() },
        //     ];
        // assert_eq!(Ok(expected_users), users);

        let u = users.get(0).unwrap();

        Ok(Human {
            id: u.id.to_string(),
            name: u.name.to_string(),
            appears_in: vec![Episode::NewHope],
            home_planet: "".to_string(),
        })
    }
}

pub struct MutationRoot;

#[graphql_object(context = GraphQLContext)]
impl MutationRoot {
    fn create_human(new_human: NewHuman) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_string(),
            name: new_human.name,
            appears_in: new_human.appears_in,
            home_planet: new_human.home_planet,
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
                id: "123".to_string(),
                name: "name".to_string(),
                appears_in: vec![Episode::Jedi],
                home_planet: "home".to_string(),
            })
        };
        Box::pin(stream)
    }

}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, Subscription>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, Subscription)
}
