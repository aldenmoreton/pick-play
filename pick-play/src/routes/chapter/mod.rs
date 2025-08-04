use axum::routing::MethodRouter;
use axum::{
    handler::Handler as _,
    middleware,
    routing::{get, post},
    Extension, Router,
};

use crate::{auth, db, AppStateRef};

use super::book;

pub mod admin;
pub mod create;
pub mod page;

async fn get_chapter_home(
    auth_session: auth::AuthSession,
    Extension(book_subscription): Extension<db::book::BookSubscription>,
    Extension(chapter): Extension<db::chapter::Chapter>,
) -> impl axum::response::IntoResponse {
    if chapter.is_open {
        page::open_book(auth_session, &book_subscription, &chapter).await
    } else {
        page::closed_book(auth_session, &book_subscription, &chapter).await
    }
}

#[inline]
pub fn router() -> Router<AppStateRef> {
    let chapter_home_page = MethodRouter::new()
        .get(get_chapter_home)
        .post(page::submit.layer(middleware::from_fn(mw::confirm_chapter_open)))
        .layer(middleware::from_fn(mw::confirm_user_access));

    Router::new()
        .nest(
            "/{chapter_id}/",
            Router::new()
                .nest(
                    "/admin/",
                    Router::new()
                        .route("/", get(admin::get).post(admin::post).delete(admin::delete))
                        .route("/user-input", get(admin::user_input))
                        .route("/open", post(admin::open))
                        .route("/visible", post(admin::visible))
                        .route("/unsubmitted-users", get(admin::unsubmitted_users)),
                )
                .route_layer(middleware::from_fn(book::mw::require_admin))
                .route("/", chapter_home_page)
                .route_layer(middleware::from_fn(mw::chapter_ext)),
        )
        .nest(
            "/create/",
            Router::new()
                .route("/", get(create::get).post(create::post))
                .route("/add", get(create::add_event))
                .route("/team-select", post(create::team_select))
                .route_layer(middleware::from_fn(book::mw::require_admin)),
        )
}

pub mod mw {
    use axum::{
        body::Body,
        extract::{Path, Request},
        http::Response,
        middleware::Next,
        response::{ErrorResponse, Redirect},
        Extension,
    };

    use crate::{
        auth::{AuthSession, BackendPgDB},
        db::chapter::get_chapter,
    };

    #[derive(serde::Deserialize)]
    pub struct ChapterIdPath {
        book_id: i32,
        chapter_id: i32,
    }

    pub async fn chapter_ext(
        auth_session: AuthSession,
        Path(ChapterIdPath {
            chapter_id,
            book_id: _b,
        }): Path<ChapterIdPath>,
        mut request: Request,
        next: Next,
    ) -> Result<Response<Body>, ErrorResponse> {
        let BackendPgDB(pool) = auth_session.backend;

        let chapter = get_chapter(chapter_id, &pool)
            .await
            .map_err(|_| Redirect::to("/"))?;

        request.extensions_mut().insert(chapter);

        Ok(next.run(request).await)
    }

    pub(super) async fn confirm_user_access(
        Extension(chapter): Extension<crate::db::chapter::Chapter>,
        Extension(book_subscription): Extension<crate::db::book::BookSubscription>,
        request: Request,
        next: axum::middleware::Next,
    ) -> Result<Response<Body>, ErrorResponse> {
        match book_subscription.role {
            crate::db::book::BookRole::Owner | crate::db::book::BookRole::Admin => {
                Ok(next.run(request).await)
            }
            crate::db::book::BookRole::Participant if chapter.is_visible => {
                Ok(next.run(request).await)
            }
            crate::db::book::BookRole::Guest {
                chapter_ids: guest_chapter_ids,
            } if chapter.is_visible && guest_chapter_ids.contains(&chapter.chapter_id) => {
                Ok(next.run(request).await)
            }
            _ => Err((axum::http::StatusCode::UNAUTHORIZED, Redirect::to("/")).into()),
        }
    }

    pub(super) async fn confirm_chapter_open(
        Extension(chapter): Extension<crate::db::chapter::Chapter>,
        request: Request,
        next: axum::middleware::Next,
    ) -> Result<Response<Body>, ErrorResponse> {
        if chapter.is_open {
            Ok(next.run(request).await)
        } else {
            Err(crate::AppNotification(
                axum::http::StatusCode::LOCKED,
                "This chapter is closed".into(),
            )
            .into())
        }
    }
}
