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
                div class="flex justify-center" {
                    fieldset class="w-1/2 border border-orange-600" {
                        legend class="ml-3" { "Admin Section" }
                        a href="chapter/create/" {
                            button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Create New Chapter"
                            }
                        }
                        br;
                        a href="admin/" {
                            button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                                "Admin"
                            }
                        }

                        (chapter_list::m(book_subscription.id, chapters.iter().filter(|c| !c.is_visible).peekable(),Some("No Unpublished Chapters")))
                    }
                }
            }

            div class="flex items-center justify-center" {
                details class="flex items-center w-max" {
                    summary class="p-3 my-1 align-middle bg-green-500 rounded-lg shadow-md select-none" {
                        "Leaderboard"
                    }
                    div hx-get="leaderboard" hx-trigger="load" hx-swap="outerhtml" class="flex items-center" {
                        "Loading..."
                    }
                }
            }

            @if let Some(guest_chapters) = guest_chapters {
                (chapter_list::m(book_subscription.id, chapters.iter().filter(|c| c.is_visible && guest_chapters.contains(&c.id)).peekable(), None))
            } @else {
                (chapter_list::m(book_subscription.id, chapters.iter().filter(|c| c.is_visible).peekable(), None))
            }
        }),
        None,
    )
}
