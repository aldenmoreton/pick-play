use axum::{
    handler::Handler as _,
    middleware,
    response::Redirect,
    routing::{get, post},
    Extension, Router,
};
use reqwest::StatusCode;

use crate::{auth, db, AppNotification, AppStateRef};

use super::book;

pub mod admin;
pub mod create;
pub mod page;

#[inline]
pub fn router() -> Router<AppStateRef> {
    let chapter_home_page = get(
        async |auth_session: auth::AuthSession,
               Extension(book_subscription): Extension<db::book::BookSubscription>,
               Extension(chapter): Extension<db::chapter::Chapter>| {
            if chapter.is_open {
                page::open_book(auth_session, &book_subscription, &chapter).await
            } else {
                page::closed_book(auth_session, &book_subscription, &chapter).await
            }
        },
    )
    .post(page::submit.layer(middleware::from_fn(
        async |Extension(chapter): Extension<db::chapter::Chapter>,
               request,
               next: middleware::Next| {
            if chapter.is_open {
                Ok(next.run(request).await)
            } else {
                Err(AppNotification(
                    StatusCode::LOCKED,
                    "This chapter is closed".into(),
                ))
            }
        },
    )))
    .layer(middleware::from_fn(
        async |Extension(chapter): Extension<db::chapter::Chapter>,
               Extension(book_subscription): Extension<db::book::BookSubscription>,
               request,
               next: middleware::Next| {
            match book_subscription.role {
                db::book::BookRole::Owner | db::book::BookRole::Admin => {
                    Ok(next.run(request).await)
                }
                db::book::BookRole::Participant if chapter.is_visible => {
                    Ok(next.run(request).await)
                }
                db::book::BookRole::Guest { chapter_ids }
                    if chapter.is_visible && chapter_ids.contains(&chapter.chapter_id) =>
                {
                    Ok(next.run(request).await)
                }
                _ => Err((StatusCode::UNAUTHORIZED, Redirect::to("/"))),
            }
        },
    ));

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
}
