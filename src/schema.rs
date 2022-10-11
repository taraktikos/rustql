use juniper::{EmptySubscription, FieldResult, graphql_object, RootNode};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

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

#[graphql_object]
impl QueryRoot {
    fn human(_id: String) -> FieldResult<Human> {
        Ok(Human {
            id: "123".to_string(),
            name: "Humanoid".to_string(),
            appears_in: vec![Episode::NewHope],
            home_planet: "".to_string(),
        })
    }
}

pub struct MutationRoot;

#[graphql_object]
impl MutationRoot {
    fn crate_human(new_human: NewHuman) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_string(),
            name: new_human.name,
            appears_in: new_human.appears_in,
            home_planet: new_human.home_planet,
        })
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}
