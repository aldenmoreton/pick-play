// TODO: Refactor some routes to end with / so that they can more
// Simply route to the pages under them
use {
    crate::routes::*,
    auth::BackendPgDB,
    axum::{
        response::{Html, IntoResponse},
        routing::get,
        Router,
    },
    axum_ctx::{RespErr, StatusCode},
    tower_http::services::ServeDir,
};

pub mod auth;

pub mod routes {
    pub mod book;
    pub mod chapter;
    pub mod finish_signup;
    pub mod home;
    pub mod session;
    pub mod signup;
    pub mod team;
}

pub mod db {
    pub mod book;
    pub mod chapter;
    pub mod event;
    pub mod spread;
    pub mod team;
    pub mod user_input;
}

pub mod templates;

type AppStateRef = &'static AppState;
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub requests: reqwest::Client,
    pub turnstile: TurnstileState,
    pub google: GoogleState,
}

pub struct TurnstileState {
    pub site_key: String,
    pub client: cf_turnstile::TurnstileClient,
}

pub struct GoogleState {
    pub redirect_url: String,
    pub oauth: oauth2::Client<
        oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
        oauth2::StandardTokenIntrospectionResponse<
            oauth2::EmptyExtraTokenFields,
            oauth2::basic::BasicTokenType,
        >,
        oauth2::StandardRevocableToken,
        oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
        oauth2::EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointSet,
    >,
}

pub fn router() -> Router<AppStateRef> {
    let site_admin_routes =
        Router::new().route("/", get(async || Html("<p>You're on the admin page</p>")));

    Router::new()
        .nest("/admin", site_admin_routes)
        .nest("/book", book::router())
        .merge(home::router())
        .route("/team-search", get(team::search::search))
        // ------------------^ Logged in Routes ^------------------
        .route_layer(axum_login::login_required!(
            BackendPgDB,
            login_url = "/login"
        ))
        .nest_service("/public", ServeDir::new("public"))
        .merge(session::router())
        .fallback(get((StatusCode::NOT_FOUND, "Could not find your route"))) // TODO: Add funny status page
}

#[derive(Debug, thiserror::Error)]
pub enum AppError<'a> {
    #[error("No Backend User")]
    BackendUser,
    #[error("Unauthorized: {0}")]
    Unauthorized(&'a str),
    #[error("Parsing: {0}")]
    Parse(&'a str),
    #[error("Database Error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl From<AppError<'_>> for RespErr {
    fn from(value: AppError) -> Self {
        match &value {
            AppError::BackendUser => {
                RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(value.to_string())
            }
            AppError::Unauthorized(_) => RespErr::new(StatusCode::UNAUTHORIZED)
                .user_msg(value.to_string())
                .log_msg(value.to_string()),
            AppError::Parse(_) => RespErr::new(StatusCode::BAD_REQUEST)
                .user_msg(value.to_string())
                .log_msg(value.to_string()),
            AppError::Sqlx(_) => {
                RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(value.to_string())
            }
        }
    }
}

impl axum::response::IntoResponse for AppError<'_> {
    fn into_response(self) -> axum::response::Response {
        RespErr::from(self).into_response()
    }
}

pub struct AppNotification(StatusCode, String);

impl axum::response::IntoResponse for AppNotification {
    fn into_response(self) -> axum::response::Response {
        (
            self.0,
            [("HX-Retarget", "body"), ("HX-Reswap", "beforeend")],
            maud::html! {
                script {
                    "alertify.set('notifier', 'position', 'top-center');"
                    @if self.0.is_success() {
                        "alertify.success("(maud::PreEscaped("\"")) (maud::PreEscaped(self.1)) (maud::PreEscaped("\""))");"
                    } @else if self.0.is_server_error() {
                        "alertify.error('Our Fault! Please Try Again.');"
                    } @else {
                        "alertify.error("(maud::PreEscaped("\"")) (maud::PreEscaped(self.1)) (maud::PreEscaped("\""))");"
                    }
                }
            },
        )
            .into_response()
    }
}

impl From<RespErr> for AppNotification {
    fn from(value: RespErr) -> Self {
        let text = value.to_string();
        let status = value.status_code;

        let _ = value.into_response();

        AppNotification(status, text)
    }
}

impl From<AppError<'_>> for AppNotification {
    fn from(value: AppError) -> Self {
        AppNotification::from(RespErr::from(value))
    }
}
