use std::{error::Error, sync::Arc};

use async_graphql::{EmptySubscription, MergedObject, Schema};

use crate::Context;

#[derive(Default)]
pub struct UsersQuery {}

#[derive(Default)]
pub struct UsersMutation {}

#[derive(MergedObject, Default)]
pub struct Query(UsersQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(UsersMutation);

pub type GraphQLSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema(ctx: Arc<Context>) -> Result<GraphQLSchema, Box<dyn Error>> {
    // Inject the initialized seervices into the Schema instance
    Ok(
        Schema::build(Query::default(), Mutation::default(), EmptySubscription)
            .data(ctx.oso.clone())
            .finish(),
    )
}
