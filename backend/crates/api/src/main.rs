mod auth;
mod routes;
mod state;

use std::sync::Arc;

use axum::http::{HeaderName, HeaderValue, Method, header};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use vd_common::Config;

use crate::state::AppState;

const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    let config = Config::from_env()?;
    let pool = vd_db::connect(&config.database_url).await?;
    vd_db::migrate(&pool).await?;

    // `vd-api create-user <email> <name> <role> <password>` bootstraps the first login.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("create-user") {
        return create_user(&pool, &args[1..]).await;
    }

    vd_jobs::spawn(pool.clone(), &config.sms);
    routes::register_prompts(&pool).await;

    let state = AppState {
        pool,
        claude: vd_ai::ClaudeClient::new(config.claude.clone()),
        config: Arc::new(config.clone()),
    };

    let mut app = routes::router(state)
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(20 * 1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(60),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(REQUEST_ID))
        .layer(SetRequestIdLayer::new(REQUEST_ID, MakeRequestUuid));

    // In production the SPA is served from the same origin by Nginx, so CORS is off.
    if let Some(origin) = &config.cors_origin {
        app = app.layer(
            CorsLayer::new()
                .allow_origin(origin.parse::<HeaderValue>()?)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        );
    }

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

async fn create_user(
    pool: &vd_db::MySqlPool,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let [email, name, role, password] = args else {
        return Err(
            "usage: vd-api create-user <email> <name> <doctor|assistant|admin> <password>".into(),
        );
    };
    let role =
        vd_common::auth::Role::parse(role).ok_or("role must be doctor, assistant or admin")?;
    let clinic_id = vd_db::users::ensure_default_clinic(pool, "My Chamber").await?;
    let hash = vd_common::auth::hash_password(password)?;
    let id = vd_db::users::create(pool, clinic_id, name, email, &hash, role.as_str()).await?;
    println!("created user {id} ({email}) in clinic {clinic_id}");
    Ok(())
}
