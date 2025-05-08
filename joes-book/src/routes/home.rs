use axum::{
    response::ErrorResponse,
    routing::{get, post},
    Router,
};

use crate::{
    auth::{authz::has_perm, AuthSession},
    db::book::get_books,
    AppError, AppStateRef,
};

use super::session;

#[inline]
pub fn router() -> Router<AppStateRef> {
    Router::new()
        .route("/logout", post(session::logout))
        .route("/", get(handler))
}

pub async fn handler(session: AuthSession) -> Result<maud::Markup, ErrorResponse> {
    let user = session.user.ok_or(AppError::BackendUser)?;

    let crate::auth::BackendPgDB(pool) = session.backend;
    let books = get_books(user.id, &pool).await?;

    let is_admin = has_perm("admin", user.id, &pool).await.unwrap_or(false);

    Ok(crate::templates::home_page::markup(
        &user.username,
        is_admin,
        books,
    ))
}
