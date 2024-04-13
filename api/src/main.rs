mod errors;
mod models;
mod routes;

use std::sync::Arc;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_dynamodb::Client;
use axum::{response::IntoResponse, Router};
use errors::APIError;
use lambda_http::{run, Error};
use radiojournal::crud::{station::CRUDStation, Context};
use tower_http::compression::CompressionLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use routes::APIDoc;

const LOCALSTACK_ENDPOINT: &str = "http://localhost:4566";

/// If LOCALSTACK environment variable is true, use LocalStack endpoints.
/// You can use your own method for determining whether to use LocalStack endpoints.
fn use_localstack() -> bool {
    std::env::var("LOCALSTACK").unwrap_or_default() == "true"
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // If you use API Gateway stages, the Rust Runtime will include the stage name
    // as part of the path that your application receives.
    // Setting the following environment variable, you can remove the stage from the path.
    // This variable only applies to API Gateway stages,
    // you can remove it if you don't use them.
    // i.e with: `GET /test-stage/todo/id/123` without: `GET /todo/id/123`
    std::env::set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    if use_localstack() {
        tracing_subscriber::fmt().compact().init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .without_time()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");

    let mut config = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
    if use_localstack() {
        config = config.endpoint_url(LOCALSTACK_ENDPOINT);
    };

    let config = config.load().await;
    let db_client = Client::new(&config);
    let table_name = std::env::var("DB_TABLE_NAME").expect("env DB_TABLE_NAME to be set");

    let context = Arc::new(Context::new(db_client, table_name));
    let crud_station = Arc::new(CRUDStation::new(context));

    let compression_layer: CompressionLayer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    let app = Router::new()
        .nest("/v1", routes::get_router().with_state(crud_station))
        .merge(SwaggerUi::new("/apidocs").url("/openapi/v1.json", APIDoc::openapi()))
        .layer(compression_layer)
        .fallback(handle_404);

    run(app).await
}

async fn handle_404() -> impl IntoResponse {
    APIError::NotFound
}
