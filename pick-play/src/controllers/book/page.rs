use axum::{extract::State, Extension};

use crate::{
    auth::{AuthSession, BackendPgDB},
    model::{
        book::{BookRole, BookSubscription},
        chapter::chapters_with_stats,
    },
    AppError, AppStateRef,
};

pub async fn book_page(
    auth_session: AuthSession,
    Extension(book_subscription): Extension<BookSubscription>,
) -> Result<maud::Markup, AppError<'static>> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let chapters = chapters_with_stats(user.id, book_subscription.id, &pool).await?;
    let guest_chapters = if let BookRole::Guest { chapter_ids } = book_subscription.role.clone() {
        Some(chapter_ids)
    } else {
        None
    };

    Ok(crate::view::book::page::m(
        user,
        book_subscription,
        chapters,
        guest_chapters,
    ))
}

pub async fn leaderboard(
    State(state): State<AppStateRef>,
    book_subscription: Extension<BookSubscription>,
) -> Result<maud::Markup, AppError<'static>> {
    let pool = &state.pool;

    let rankings = crate::model::book::leaderboard(book_subscription.id, pool).await?;

    Ok(maud::html! {
        div class="flex justify-center w-full" {
            table class="w-auto max-w-md text-sm" {
                thead class="text-xs text-gray-700 uppercase bg-green-400" {
                    tr {
                        th scope="col" class="px-6 py-3" { "Rank" }
                        th scope="col" class="px-6 py-3" { "User" }
                        th scope="col" class="px-6 py-3" { "Total Points" }
                    }
                }

                tbody {
                    @for (i, rank) in rankings.iter().enumerate() {
                        tr.text-blue-500[rank.username == "Guests"] class="bg-white" {
                            @if rank.rank == i as i32 + 1 {
                                td class="px-6 py-4" {(i + 1)}
                            } @else {
                                td {}
                            }
                            td class="px-6 py-4" {
                                (rank.username)
                                br;
                                @if rank.added_points > 0 {
                                    span class="text-red-500" {"Added Points: "(rank.added_points)}
                                }
                            }
                            td class="px-6 py-4" {(rank.total_points)}
                        }
                    }
                }
            }
        }
    })
}
