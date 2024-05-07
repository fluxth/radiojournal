mod errors;
mod extractors;
mod models;
mod routes;

use std::{sync::Arc, time::Duration};

use axum::{
    http::Request,
    response::{IntoResponse, Response},
    Router,
};
use errors::APIError;
use lambda_http::{request::RequestContext, run, Error, RequestExt};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{info, info_span, Span};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use radiojournal::{
    crud::{play::CRUDPlay, station::CRUDStation, track::CRUDTrack},
    init,
};
use routes::APIDoc;

struct AppState {
    crud_play: CRUDPlay,
    crud_track: CRUDTrack,
    crud_station: CRUDStation,
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

    if init::use_localstack() {
        tracing_subscriber::fmt().compact().init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .without_time()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let context = init::initialize()
        .await
        .expect("initialize radiojournal app");

    let crud_play = CRUDPlay::new(context.clone());
    let crud_track = CRUDTrack::new(context.clone());
    let crud_station = CRUDStation::new(context);

    let app_state = Arc::new(AppState {
        crud_play,
        crud_track,
        crud_station,
    });

    let compression_layer: CompressionLayer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let (client_ip, user_agent) =
                if let RequestContext::ApiGatewayV2(request_context) = request.request_context() {
                    (
                        request_context.http.source_ip,
                        request_context.http.user_agent,
                    )
                } else {
                    (None, None)
                };

            info_span!(
                "http",
                method = ?request.method(),
                path = request.raw_http_path(),
                client_ip = client_ip,
                user_agent = user_agent
            )
        })
        .on_response(|response: &Response, latency: Duration, _span: &Span| {
            let latency_ms: u64 = latency.as_millis().try_into().unwrap_or(u64::MAX);
            info!(
                event = "request-log",
                latency_ms = latency_ms,
                response_status = response.status().as_u16(),
            );
        });

    let app = Router::new()
        .nest("/v1", routes::get_router().with_state(app_state))
        .merge(SwaggerUi::new("/apidocs").url("/openapi/v1.json", APIDoc::openapi()))
        .layer(compression_layer)
        .fallback(handle_404)
        .layer(trace_layer);

    run(app).await
}

async fn handle_404() -> impl IntoResponse {
    APIError::NotFound
}
