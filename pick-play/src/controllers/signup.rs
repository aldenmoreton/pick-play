use axum::{extract::State, response::IntoResponse, Form};
use axum_ctx::{RespErrCtx, RespErrExt, StatusCode};

use crate::{auth::AuthSession, view, AppError, AppNotification, AppStateRef};

pub async fn signup_page(state: State<AppStateRef>) -> maud::Markup {
    view::signup::m(&state.turnstile.site_key)
}

#[derive(serde::Deserialize)]
pub struct SignUpForm {
    username: String,
    password: String,
    password_confirmation: String,
    #[serde(rename = "cf-turnstile-response")]
    pub turnstile_response: String,
}

pub async fn signup_form(
    mut auth_session: AuthSession,
    State(state): State<AppStateRef>,
    Form(form): Form<SignUpForm>,
) -> Result<impl IntoResponse, AppNotification> {
    let cf_validate: Result<cf_turnstile::SiteVerifyResponse, cf_turnstile::error::TurnstileError> =
        state
            .turnstile
            .client
            .siteverify(cf_turnstile::SiteVerifyRequest {
                response: form.turnstile_response,
                ..Default::default()
            })
            .await;

    tracing::debug!("{cf_validate:?}");

    if !cf_validate.map(|v| v.success).unwrap_or(false) {
        return Err(AppNotification(
            StatusCode::UNAUTHORIZED,
            "You did not pass our check for robots".into(),
        ));
    }

    if form.username.is_empty()
        || form
            .username
            .chars()
            .any(|c| c.is_whitespace() || !c.is_ascii_alphanumeric())
    {
        return Err(AppNotification(
            StatusCode::BAD_REQUEST,
            "Username is not allowed".into(),
        ));
    }

    let pool = &state.pool;

    if form.password != form.password_confirmation {
        return Err(AppNotification(
            StatusCode::CONFLICT,
            "Password does not match confirmation".into(),
        ));
    }

    let user_exists = crate::model::user::user_exists(&form.username, pool)
        .await
        .map_err(AppError::from)?;

    if user_exists {
        return Err(AppNotification(
            StatusCode::CONFLICT,
            "Username already taken".into(),
        ));
    }

    let user = auth_session
        .backend
        .signup(&form.username.to_lowercase(), &form.password)
        .await
        .map_err(AppError::from)?;

    auth_session
        .login(&user)
        .await
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .user_msg("Could not log in")?;

    Ok([("HX-Location", "/")].into_response())
}
