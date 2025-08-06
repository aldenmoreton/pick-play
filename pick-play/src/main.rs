use axum_login::AuthManagerLayerBuilder;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
pub async fn shuttle(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:postgres@localhost:5432/pick-play"
    )]
    pool: sqlx::PgPool,
) -> shuttle_axum::ShuttleAxum {
    let auth_layer = {
        let backend = pick_play::controllers::auth::BackendPgDB(pool.clone());
        backend.init_admin().await.ok();

        let session_store = PostgresStore::new(pool.clone());
        session_store
            .migrate()
            .await
            .expect("Could not migrate database");
        let session_layer = SessionManagerLayer::new(session_store)
            .with_same_site(tower_sessions::cookie::SameSite::Lax)
            .with_name("book_session")
            // .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::weeks(2)));

        AuthManagerLayerBuilder::new(backend, session_layer).build()
    };

    let state: pick_play::AppState = {
        let turnstile_site_key: String = secrets
            .get("TURNSTILE_SITE_KEY")
            .unwrap_or_else(|| "1x00000000000000000000AA".into());

        let turnstile_secret = secrets
            .get("TURNSTILE_SECRET_KEY")
            .unwrap_or_else(|| "1x0000000000000000000000000000000AA".into());

        let google_redirect_url = secrets
            .get("GOOGLE_OAUTH_REDIRECT")
            .unwrap_or("http://localhost:8000/api/auth/google".to_string());

        let google_oauth = oauth2::basic::BasicClient::new(oauth2::ClientId::new(
            secrets
                .get("GOOGLE_OAUTH_CLIENT_ID")
                .expect("GOOGLE_OAUTH_CLIENT_ID must be set in secrets"),
        ))
        .set_token_uri(
            oauth2::TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".into())
                .expect("Failed to create OAuth token URL"),
        )
        .set_auth_uri(
            oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
                .expect("Failed to create OAuth auth URL"),
        )
        .set_client_secret(oauth2::ClientSecret::new(
            secrets
                .get("GOOGLE_OAUTH_SECRET")
                .expect("GOOGLE_OAUTH_SECRET must be set in secrets"),
        ))
        .set_redirect_uri(
            oauth2::RedirectUrl::new(google_redirect_url.clone())
                .expect("Failed to create OAuth redirect URL"),
        );

        pick_play::AppState {
            pool,
            requests: reqwest::Client::new(),
            turnstile: pick_play::TurnstileState {
                site_key: turnstile_site_key,
                client: cf_turnstile::TurnstileClient::new(turnstile_secret.into()),
            },
            google: pick_play::GoogleState {
                redirect_url: google_redirect_url,
                oauth: google_oauth,
            },
        }
    };

    let app = pick_play::router()
        .layer(auth_layer)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(&*Box::leak(Box::new(state)));

    Ok(shuttle_axum::AxumService(app))
}

#[cfg(not(feature = "shuttle"))]
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL ENV");

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Could not make pool.");

    let auth_layer = {
        let backend = pick_play::controllers::auth::BackendPgDB(pool.clone());
        backend.init_admin().await.ok();

        let session_store = PostgresStore::new(pool.clone());
        session_store
            .migrate()
            .await
            .expect("Could not migrate database");
        let session_layer = SessionManagerLayer::new(session_store)
            .with_same_site(tower_sessions::cookie::SameSite::Lax)
            .with_name("book_session")
            // .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::weeks(2)));

        AuthManagerLayerBuilder::new(backend, session_layer).build()
    };

    let state: pick_play::AppState = {
        let turnstile_site_key: String = std::env::var("TURNSTILE_SITE_KEY")
            .unwrap_or_else(|_| "1x00000000000000000000AA".into());

        let turnstile_secret = std::env::var("TURNSTILE_SECRET_KEY")
            .unwrap_or_else(|_| "1x0000000000000000000000000000000AA".into());

        let google_redirect_url = std::env::var("GOOGLE_OAUTH_REDIRECT")
            .unwrap_or("http://localhost:8000/api/auth/google".to_string());

        let google_oauth = oauth2::basic::BasicClient::new(oauth2::ClientId::new(
            std::env::var("GOOGLE_OAUTH_CLIENT_ID")
                .expect("GOOGLE_OAUTH_CLIENT_ID environment variable must be set"),
        ))
        .set_token_uri(
            oauth2::TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".into())
                .expect("Failed to create OAuth token URL"),
        )
        .set_auth_uri(
            oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
                .expect("Failed to create OAuth auth URL"),
        )
        .set_client_secret(oauth2::ClientSecret::new(
            std::env::var("GOOGLE_OAUTH_SECRET")
                .expect("GOOGLE_OAUTH_SECRET environment variable must be set"),
        ))
        .set_redirect_uri(
            oauth2::RedirectUrl::new(google_redirect_url.clone())
                .expect("Failed to create OAuth redirect URL"),
        );

        pick_play::AppState {
            pool,
            requests: reqwest::Client::new(),
            turnstile: pick_play::TurnstileState {
                site_key: turnstile_site_key,
                client: cf_turnstile::TurnstileClient::new(turnstile_secret.into()),
            },
            google: pick_play::GoogleState {
                redirect_url: google_redirect_url,
                oauth: google_oauth,
            },
        }
    };

    let app = pick_play::router()
        .layer(auth_layer)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(&*Box::leak(Box::new(state)));

    println!();
    println!("Starting server at http://localhost:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Failed to bind to address 0.0.0.0:8000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
