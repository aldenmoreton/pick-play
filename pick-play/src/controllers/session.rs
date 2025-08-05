use {
    super::finish_signup,
    crate::{
        auth::{self, AuthSession},
        AppStateRef,
    },
};
use {
    axum::{
        body::Body,
        extract::State,
        http::{Response, StatusCode},
        middleware,
        response::IntoResponse,
        routing::{get, post},
        Router,
    },
    axum_ctx::{RespErr, RespErrCtx, RespErrExt},
};

#[inline]
pub fn router() -> Router<AppStateRef> {
    Router::new()
        .route("/api/auth/google", get(google::google_oauth))
        .route(
            "/finish-signup",
            get(finish_signup::finish_page).post(finish_signup::post),
        )
        .route("/login", get(crate::session::login_page))
        .route_layer(middleware::from_fn(
            async |auth_session: auth::AuthSession, request, next: middleware::Next| {
                if auth_session.user.is_some() {
                    return axum::response::Redirect::to("/").into_response();
                }
                next.run(request).await.into_response()
            },
        ))
        .route("/logout", post(crate::session::logout))
}

pub async fn login_page(State(state): State<AppStateRef>) -> maud::Markup {
    crate::view::login::m(state)
}

pub async fn login_explaination() -> maud::Markup {
    maud::html! {
        p class="max-w-60" {
            "For security reasons, we no longer support logging in with username and password. "
            "Don't worry, you will be able to link your old account during the new login process."
        }
    }
}

pub async fn logout(mut auth_session: self::AuthSession) -> Result<Response<Body>, RespErr> {
    auth_session
        .logout()
        .await
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .log_msg("Could not log out user")
        .user_msg("Logout unsuccessful")?;

    Ok([("HX-Redirect", "/login")].into_response())
}

#[derive(Debug, serde::Deserialize)]
pub enum OauthProfile {
    #[serde(rename = "google")]
    Google(google::GoogleOauth),
}

pub mod google {
    use axum::{
        extract::{rejection::QueryRejection, Query, State},
        response::{ErrorResponse, IntoResponse, Redirect},
    };
    use axum_ctx::{RespErr, RespErrCtx, RespErrExt};
    use axum_extra::extract::CookieJar;
    use oauth2::TokenResponse as _;
    use reqwest::StatusCode;

    use crate::{auth::AuthSession, AppError};

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct GoogleOauth {
        pub sub: String,
        #[serde(flatten)]
        pub extra: std::collections::HashMap<String, serde_json::Value>,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct GoogleAuthRequest {
        code: String,
    }

    pub async fn google_oauth(
        mut auth_session: AuthSession,
        cookie_jar: CookieJar,
        State(state): State<crate::AppStateRef>,
        query: Result<Query<GoogleAuthRequest>, QueryRejection>,
    ) -> Result<impl IntoResponse, ErrorResponse> {
        let query = query
            .map_err(|e| {
                RespErr::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .log_msg(format!("Query params in google oauth redirect: {e:?}"))
            })?
            .0;

        let token = state
            .google
            .oauth
            .exchange_code(oauth2::AuthorizationCode::new(query.code))
            .request_async(&reqwest::Client::new())
            .await
            .map_err(|e| {
                RespErr::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .log_msg(format!("No way to get token: {e:?}"))
            })?;

        let profile = state
            .requests
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .bearer_auth(token.access_token().secret())
            .send()
            .await
            .map_err(|e| {
                RespErr::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .log_msg(format!("Can't get access token response: {e:?}"))
            })?
            .text()
            .await
            .map_err(|e| {
                RespErr::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .log_msg(format!("Don't understand oauth token: {e:?}"))
            })?;

        let profile: GoogleOauth = serde_json::from_str(&profile).map_err(|e| {
            RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(format!("Json no go: {e:?}"))
        })?;

        let pool = &state.pool;

        let user = sqlx::query_as!(
            crate::auth::BackendUser,
            "
            SELECT users.id, users.username, users.password as pw_hash
            FROM users
            JOIN oauth ON users.id = oauth.user_id
            WHERE oauth.sub = $1 AND oauth.provider = $2
            ",
            profile.sub,
            "google"
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

        if let Some(user) = user {
            auth_session
                .login(&user)
                .await
                .ctx(StatusCode::INTERNAL_SERVER_ERROR)
                .log_msg("Could not log in via google oauth")?;
            return Err(Redirect::to("/").into());
        }

        let content = serde_json::to_value(profile.clone())
            .map_err(|e| RespErr::new(StatusCode::INTERNAL_SERVER_ERROR).log_msg(e.to_string()))?;

        sqlx::query!(
            "
            INSERT INTO oauth(sub, provider, content)
            VALUES ($1, $2, jsonb_build_object('google', $3::JSONB))
            ON CONFLICT (sub, provider)
            DO NOTHING
            ",
            profile.sub,
            "google",
            content
        )
        .execute(pool)
        .await
        .map_err(AppError::from)?;

        let signup_token = sqlx::query!(
            "
            INSERT INTO signup_tokens(sub, provider)
            VALUES ($1, $2)
            RETURNING token
            ",
            profile.sub,
            "google"
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?
        .token;

        let cookie =
            tower_sessions::cookie::Cookie::build(("signup_token", signup_token.to_string()))
                .http_only(true)
                .same_site(tower_sessions::cookie::SameSite::Lax)
                .path("/")
                .build();

        Ok((cookie_jar.add(cookie), Redirect::to("/finish-signup")).into_response())
    }
}
