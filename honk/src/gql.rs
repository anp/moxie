use crate::state::WorkspaceState;
use actix_web::{http::header, middleware, web, App, Error, HttpResponse, HttpServer};
use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};

impl juniper::Context for WorkspaceState {}

pub struct Query;

#[graphql_object(context = WorkspaceState)]
impl Query {
    fn apiVersion() -> String {
        "1.0".to_string()
    }
    // #[graphql(arguments(id(description = "id of the user")))]
    // fn user(database: &Database, id: i32) -> Option<&User> {
    //     database.get_user(&id)
    // }
}

pub type Schema =
    RootNode<'static, Query, EmptyMutation<WorkspaceState>, EmptySubscription<WorkspaceState>>;

pub fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<WorkspaceState>::new(),
        EmptySubscription::<WorkspaceState>::new(),
    )
}

pub async fn graphiql_route() -> Result<HttpResponse, Error> {
    graphiql_handler("/graphgl", None).await
}

pub async fn playground_route() -> Result<HttpResponse, Error> {
    playground_handler("/graphgl", None).await
}

pub async fn graphql_route(
    req: actix_web::HttpRequest,
    // state: &WorkspaceState,
    payload: actix_web::web::Payload,
    schema: web::Data<Schema>,
) -> Result<HttpResponse, Error> {
    graphql_handler(&schema, todo!("get the state"), req, payload).await
}
