use std::{error::Error, net::SocketAddr, sync::Arc};

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{Html, IntoResponse},
    routing::{get, IntoMakeService},
    Extension, Router,
};
use flowist_auth::Subject;
use graphql::GraphQLSchema;
use hyper::{server::conn::AddrIncoming, Server};
use oso::Oso;
use sea_orm::DatabaseConnection;
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

pub mod graphql;

pub struct Context {
    // The database connections
    pub db: Arc<DatabaseConnection>,
    // The authorization library
    pub oso: Oso,
}

impl Context {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            oso: Oso::new(),
            db: Arc::new(sea_orm::Database::connect("").await?),
        })
    }
}

/// Handle health check requests
pub async fn health_handler() -> impl IntoResponse {
    json!({
        "code": "200",
        "success": true,
    })
    .to_string()
}

pub async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// Handle GraphQL Requests
pub async fn graphql_handler(
    Extension(schema): Extension<GraphQLSchema>,
    Extension(ctx): Extension<Arc<Context>>,
    sub: Subject,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // Retrieve the request User, if username is present
    let user = if let Subject(Some(ref username)) = sub {
        None
        // ctx.users
        //     .get_by_username(username, &true)
        //     .await
        //     .unwrap_or(None)
    } else {
        None
    };
    // Add the Subject and optional User to the context
    let request = req.into_inner().data(sub).data(None);
    schema.execute(request).await.into()
}

fn router() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/graphql", get(graphiql).post(graphql_handler))
        // We can still add middleware
        .layer(TraceLayer::new_for_http())
}

pub async fn run(
    context: Arc<Context>,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, Box<dyn Error>> {
    let port = "";

    let router = router();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum::Server::bind(&addr).serve(router.into_make_service());

    Ok(server)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let config = get_config();
    let context = Arc::new(Context::init().await?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_hello_world() {
        let app = router();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, World!");
    }

    #[tokio::test]
    async fn json() {
        let app = router();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/json")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_vec(&json!([1, 2, 3, 4])).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({ "data": [1, 2, 3, 4] }));
    }

    #[tokio::test]
    async fn not_found() {
        let app = router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(body.is_empty());
    }
}
