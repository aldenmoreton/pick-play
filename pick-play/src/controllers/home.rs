use axum::response::ErrorResponse;

use crate::{
    auth::{authz::has_perm, AuthSession},
    model::book::user_books_stats,
    AppError,
};

pub async fn handler(session: AuthSession) -> Result<maud::Markup, ErrorResponse> {
    let user = session.user.ok_or(AppError::BackendUser)?;

    let crate::auth::BackendPgDB(pool) = session.backend;
    let book_stats = user_books_stats(user.id, &pool)
        .await
        .map_err(AppError::from)?;

    let is_admin = has_perm("admin", user.id, &pool).await.unwrap_or(false);

    Ok(crate::view::home::m(&user.username, is_admin, book_stats))
}
