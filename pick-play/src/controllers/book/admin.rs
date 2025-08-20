use axum::{
    extract::{Query, State},
    response::{ErrorResponse, IntoResponse},
    Extension, Form,
};
use axum_ctx::RespErr;
use reqwest::StatusCode;

use crate::{
    auth::AuthSession,
    model::{
        book::{BookSubscription, get_book_members, search_users_not_in_book, add_user_to_book, remove_user_from_book, delete_book_cascade},
        chapter::chapters_with_stats,
    },
    AppError, AppStateRef,
};

pub async fn handler(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<maud::Markup, AppError<'static>> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let pool = &auth_session.backend.0;

    let members = get_book_members(book_subscription.id, book_subscription.user_id, pool)
        .await
        .map_err(AppError::from)?;

    let chapters = chapters_with_stats(user.id, book_subscription.id, pool).await?;
    let unpublished_chapters = chapters
        .iter()
        .filter(|chapter| !chapter.is_visible)
        .peekable();

    Ok(crate::view::book::admin::m(
        &user,
        &book_subscription,
        unpublished_chapters,
        &members,
    ))
}

#[derive(serde::Deserialize)]
pub struct AddUserParams {
    user_id: i32,
    username: String,
}

pub async fn add_user(
    State(state): State<AppStateRef>,
    Extension(book_subscription): Extension<BookSubscription>,
    user_params: Form<AddUserParams>,
) -> Result<maud::Markup, ErrorResponse> {
    let pool = &state.pool;

    add_user_to_book(user_params.user_id, book_subscription.id, pool)
        .await
        .map_err(AppError::from)?
        .ok_or(RespErr::new(StatusCode::BAD_REQUEST).user_msg("Could not find user to add"))?;

    Ok(crate::view::book::admin::new_member_row(
        user_params.user_id,
        &user_params.username,
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct UserSearchParams {
    username: String,
}

pub async fn search_user(
    State(state): State<AppStateRef>,
    Query(UserSearchParams {
        username: search_username,
    }): Query<UserSearchParams>,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<maud::Markup, AppError<'static>> {
    let pool = &state.pool;

    if search_username.is_empty() {
        return Ok(maud::html!());
    }

    let matching_users = search_users_not_in_book(&search_username, book_subscription.id, pool)
        .await
        .map_err(AppError::from)?;

    Ok(crate::view::book::admin::user_search_results(
        &matching_users,
        book_subscription.id,
    ))
}

#[derive(serde::Deserialize)]
pub struct RemoveUserForm {
    user_id: i32,
}

pub async fn remove_user(
    State(state): State<AppStateRef>,
    book: Extension<BookSubscription>,
    form: Form<RemoveUserForm>,
) -> Result<(), AppError<'static>> {
    let pool = &state.pool;

    remove_user_from_book(form.user_id, book.id, pool)
        .await
        .map_err(AppError::from)?;

    Ok(())
}

pub async fn delete(
    State(state): State<AppStateRef>,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<impl IntoResponse, AppError<'static>> {
    let pool = &state.pool;

    delete_book_cascade(book_subscription.id, pool)
        .await
        .map_err(AppError::from)?;

    Ok([("HX-Redirect", "/")].into_response())
}
