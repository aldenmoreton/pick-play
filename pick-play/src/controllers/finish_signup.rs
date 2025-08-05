use axum::{
    extract::State,
    response::{ErrorResponse, IntoResponse, Redirect},
    Form,
};
use axum_ctx::RespErr;
use reqwest::StatusCode;

use crate::{auth::AuthSession, AppError, AppNotification};

use super::session::OauthProfile;

pub async fn finish_page(
    cookie_jar: axum_extra::extract::CookieJar,
    State(state): State<crate::AppStateRef>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let pool = &state.pool;

    let signup_token = cookie_jar
        .get("signup_token")
        .ok_or(Redirect::to("/login"))?;

    let oauth_profile = sqlx::query!(
        "
		SELECT oauth.content AS content
		FROM signup_tokens
		JOIN oauth ON signup_tokens.sub = oauth.sub AND signup_tokens.provider = oauth.provider
		WHERE token = $1
		",
        signup_token.value()
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?
    .ok_or(Redirect::to("/login"))?;

    let OauthProfile::Google(profile) = serde_json::from_value(oauth_profile.content)
        .map_err(|e| RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(e.to_string()))?;

    Ok(crate::view::finish_signup::m(profile, state).into_response())
}

#[derive(serde::Deserialize)]
pub struct FinishSignupForm {
    username: String,
    #[serde(rename = "cf-turnstile-response")]
    turnstile_response: String,
}

pub async fn post(
    mut auth_session: AuthSession,
    cookie_jar: axum_extra::extract::CookieJar,
    State(state): State<crate::AppStateRef>,
    Form(form): Form<FinishSignupForm>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let cf_validate: Result<cf_turnstile::SiteVerifyResponse, cf_turnstile::error::TurnstileError> =
        state
            .turnstile
            .client
            .siteverify(cf_turnstile::SiteVerifyRequest {
                response: form.turnstile_response,
                ..Default::default()
            })
            .await;

    if !cf_validate.map(|v| v.success).unwrap_or(false) {
        return Err(AppNotification(
            StatusCode::UNAUTHORIZED,
            "You did not pass our check for robots".into(),
        )
        .into());
    }

    if form.username.is_empty()
        || form
            .username
            .chars()
            .any(|c| c.is_whitespace() || !c.is_ascii_alphanumeric())
    {
        return Err(
            AppNotification(StatusCode::BAD_REQUEST, "Username is not allowed".into()).into(),
        );
    }

    let signup_token = cookie_jar
        .get("signup_token")
        .ok_or([("HX-Redirect", "/login")])?;

    let mut transaction = state.pool.begin().await.map_err(AppError::from)?;

    let oauth_profile = sqlx::query!(
        "
        DELETE FROM signup_tokens
        WHERE token = $1
        RETURNING sub, provider
        ",
        signup_token.value()
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|e| AppNotification::from(AppError::from(e)))?
    .ok_or([("HX-Redirect", "/login")])?;

    let user = sqlx::query_as!(
        crate::auth::BackendUser,
        r#"
        INSERT INTO USERS (username)
        VALUES ($1)
        ON CONFLICT (username) DO NOTHING
        RETURNING id, username, password AS "pw_hash"
        "#,
        form.username
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(AppError::from)?
    .ok_or(AppNotification(
        StatusCode::BAD_REQUEST,
        "Username already taken".into(),
    ))?;

    sqlx::query!(
        "
        UPDATE oauth
        SET user_id = $1
        WHERE sub = $2 AND provider = $3
        ",
        user.id,
        oauth_profile.sub,
        oauth_profile.provider
    )
    .execute(&mut *transaction)
    .await
    .map_err(AppError::from)?;

    auth_session
        .login(&user)
        .await
        .map_err(|e| RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(e.to_string()))?;

    transaction.commit().await.map_err(AppError::from)?;

    Ok((cookie_jar.remove("signup_token"), [("HX-Location", "/")]).into_response())
}
