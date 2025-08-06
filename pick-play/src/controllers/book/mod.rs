use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{auth::authz, AppStateRef};

use super::chapter;

pub mod admin;
pub mod create;
pub mod page;

#[inline]
pub fn router() -> Router<AppStateRef> {
    Router::new()
        .nest(
            "/{book_id}/",
            Router::new()
                .nest(
                    "/admin/",
                    Router::new()
                        .route("/", get(admin::handler).delete(admin::delete))
                        .route("/user-search", get(admin::search_user))
                        .route("/add-user", post(admin::add_user))
                        .route("/remove-user", post(admin::remove_user)),
                )
                .route_layer(middleware::from_fn(mw::require_admin))
                .nest("/chapter/", chapter::router())
                .route("/leaderboard", get(page::leaderboard))
                .route("/", get(page::book_page)),
        )
        .route_layer(middleware::from_fn(mw::require_member))
        .route(
            "/create",
            post(create::handler).layer(middleware::from_fn(authz::mw::require_site_admin)),
        )
}

pub mod mw {
    use axum::{
        body::Body,
        extract::{Path, Request},
        http::{Response, StatusCode},
        middleware::Next,
        response::{ErrorResponse, IntoResponse, Redirect},
        Extension,
    };
    use axum_ctx::RespErr;

    use crate::{
        auth::{AuthSession, BackendPgDB},
        model::book::{get_book, BookRole, BookSubscription},
        AppError,
    };

    #[derive(serde::Deserialize)]
    pub struct BookIdPath {
        book_id: i32,
    }

    pub async fn require_member(
        Path(BookIdPath { book_id }): Path<BookIdPath>,
        auth_session: AuthSession,
        mut request: Request,
        next: Next,
    ) -> Result<Response<Body>, ErrorResponse> {
        let user = auth_session.user.ok_or(AppError::BackendUser)?;
        let BackendPgDB(pool) = auth_session.backend;

        let book_subscription = match get_book(user.id, book_id, &pool).await {
            Ok(BookSubscription {
                role: BookRole::Unauthorized,
                ..
            }) => return Err((StatusCode::UNAUTHORIZED, Redirect::to("/")).into()),
            Err(e) => {
                let _ = RespErr::new(StatusCode::NOT_FOUND)
                    .user_msg("Could not find book")
                    .log_msg(e.to_string())
                    .into_response();
                return Err(Redirect::to("/").into());
            }
            Ok(subscription) => subscription,
        };

        request.extensions_mut().insert(book_subscription);
        Ok(next.run(request).await)
    }

    pub async fn require_admin(
        Extension(book_subscription): Extension<BookSubscription>,
        request: Request,
        next: Next,
    ) -> Result<Response<Body>, RespErr> {
        if book_subscription.role != BookRole::Admin {
            return Err(
                AppError::Unauthorized("You do not have admin privilages for this book").into(),
            );
        }

        Ok(next.run(request).await)
    }
}
