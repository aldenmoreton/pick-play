use crate::{
    auth::BackendUser,
    model::{
        book::{BookRole, BookSubscription},
        chapter::ChapterStats,
    },
    view::chapter::list as chapter_list,
};

pub fn m(
    user: BackendUser,
    book_subscription: BookSubscription,
    chapters: Vec<ChapterStats>,
    guest_chapters: Option<Vec<i32>>,
) -> maud::Markup {
    super::authenticated(
        &user.username,
        Some(&book_subscription.name),
        None,
        None,
        Some(maud::html! {
            p {
                a href="/" class="text-blue-400 hover:underline" {"Home"} " > "
                a { (book_subscription.name) }
            }
        }),
        Some(maud::html! {
            h1 class="text-4xl font-extrabold" {(book_subscription.name)}
            @if book_subscription.role == BookRole::Admin {
                a href="admin/" {
                    button class="fixed z-50 px-3 py-2 text-sm font-bold text-white transition-colors bg-orange-600 rounded-full shadow-lg bottom-4 right-4 hover:bg-orange-700" {
                        "Admin"
                    }
                }
            }

            div class="flex items-center justify-center w-full" {
                details class="relative w-auto" {
                    summary class="p-3 my-1 align-middle bg-green-500 rounded-lg shadow-md cursor-pointer select-none" {
                        "Leaderboard"
                    }
                    div hx-get="leaderboard" hx-trigger="load" hx-swap="outerhtml" class="w-full mt-2 bg-white border border-gray-300 rounded-lg shadow-lg" {
                        "Loading..."
                    }
                }
            }

            @if let Some(guest_chapters) = guest_chapters {
                (chapter_list::m(book_subscription.id, chapters.iter().filter(|c| c.is_visible && guest_chapters.contains(&c.id)).peekable(), None))
            } @else if book_subscription.role == BookRole::Admin {
                (chapter_list::m(book_subscription.id, chapters.iter().peekable(), None))
            }   @else {
                (chapter_list::m(book_subscription.id, chapters.iter().filter(|c| c.is_visible).peekable(), None))
            }
        }),
        None,
    )
}
