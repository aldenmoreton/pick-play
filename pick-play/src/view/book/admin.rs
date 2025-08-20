use std::iter::Peekable;

use crate::{
    auth::BackendUser,
    model::{
        book::{BookMember, BookSubscription},
        chapter::ChapterStats,
    },
};

pub fn m<'a, I>(
    user: &BackendUser,
    book_subscription: &BookSubscription,
    unpublished_chapters: Peekable<I>,
    members: &[BookMember],
) -> maud::Markup
where
    I: Iterator<Item = &'a ChapterStats>,
{
    crate::view::authenticated(
        &user.username,
        Some(format!("{} - Admin", book_subscription.name).as_str()),
        None,
        None,
        Some(maud::html! {
            p {
                a href="/" class="text-blue-400 hover:underline" {"Home"} " > "
                a href=".." class="text-blue-400 hover:underline" { (book_subscription.name) } " > "
                a {"Admin"}
            }
        }),
        Some(maud::html! {
            div class="flex flex-col items-center justify-center" {
                (create_chapter_button())
                (chapter_management_section(book_subscription.id, unpublished_chapters))
                (danger_zone())
                (member_management_table(user, members))
            }
        }),
        None,
    )
}

fn create_chapter_button() -> maud::Markup {
    maud::html! {
        a href="../chapter/create/" {
            button class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                "Create New Chapter"
            }
        }
    }
}

fn chapter_management_section<'a, I>(
    book_id: i32,
    unpublished_chapters: Peekable<I>,
) -> maud::Markup
where
    I: Iterator<Item = &'a ChapterStats>,
{
    maud::html! {
        div class="flex justify-center mb-6" {
            fieldset class="w-1/2 border border-orange-600" {
                legend class="ml-3" { "Chapter Management" }
                (crate::view::chapter::list::m(book_id, unpublished_chapters, Some("No Unpublished Chapters")))
            }
        }
    }
}

fn danger_zone() -> maud::Markup {
    maud::html! {
        details {
            summary {
                span class="text-red-500" {"Danger Zone"}
            }
            button
                hx-delete="."
                hx-confirm="Are you sure you wish to delete this book, all chapters, and all picks within FOREVER?"
                class="p-0.5 font-bold text-white bg-red-600 rounded hover:bg-red-700" {
                "Delete Book"
            }
        }
    }
}

fn member_management_table(user: &BackendUser, members: &[BookMember]) -> maud::Markup {
    maud::html! {
        div class="relative mt-5 overflow-x-auto rounded-lg" {
            table class="w-full text-sm text-left text-gray-500 rtl:text-right" {
                (table_header())
                (table_body(user, members))
                (table_footer())
            }
        }
    }
}

fn table_header() -> maud::Markup {
    maud::html! {
        thead class="text-xs text-gray-700 uppercase bg-gray-100" {
            tr {
                th scope="col" class="px-6 py-3 rounded-s-lg" { "username" }
                th scope="col" class="px-6 py-3" { "status" }
                th scope="col" class="px-6 py-3 rounded-e-lg" { "action" }
            }
        }
    }
}

fn table_body(user: &BackendUser, members: &[BookMember]) -> maud::Markup {
    maud::html! {
        tbody {
            (admin_row(&user.username))
            @for member in members {
                (member_row(member))
            }
        }
    }
}

fn admin_row(username: &str) -> maud::Markup {
    maud::html! {
        tr class="bg-white" {
            td scope="row" class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap" { (username) }
            td class="px-6 py-4" { "admin" }
            td class="px-6 py-4" {
                button {
                    "Heavy is The Head" br;
                    "That Wears The Crown"
                }
            }
        }
    }
}

fn member_row(member: &BookMember) -> maud::Markup {
    maud::html! {
        tr class="bg-white" hx-target="this" {
            td class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap" { (member.username) }
            td class="px-6 py-4" { (member.role) }
            td class="px-6 py-4" {
                button
                    hx-post="remove-user"
                    hx-vals={r#"{"user_id":""#(member.id)r#""}"#}
                    class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                    "Remove"
                }
            }
        }
    }
}

fn table_footer() -> maud::Markup {
    maud::html! {
        tfoot {
            tr class="font-semibold text-gray-900 bg-green-400" {
                th scope="row" class="px-6 py-3 text-base" { "Add Member" }
                th colspan="2" {
                    input
                        name="username"
                        hx-get="user-search"
                        hx-trigger="input changed delay:200ms, search"
                        hx-target="next ul"
                        type="search"
                        autocomplete="off"
                        placeholder="username"
                        class="border border-green-300";
                   ul {}
                }
            }
        }
    }
}

pub fn user_search_results(
    users: &[crate::model::book::UserSearchResult],
    book_id: i32,
) -> maud::Markup {
    maud::html!(
        @for user in users {
            li {
                button
                    name="username"
                    value=(user.username)
                    hx-post={"/book/"(book_id)"/admin/add-user"}
                    hx-vals={r#"{"user_id":""#(user.id)r#""}"#}
                    hx-target="previous tbody"
                    hx-on-click=r#"document.querySelector('input[type="search"]').value=""; document.querySelector('ul').innerHTML="";"#
                    hx-swap="beforeend" {
                        (user.username)
                    }
            }
        }
    )
}

pub fn new_member_row(user_id: i32, username: &str) -> maud::Markup {
    maud::html! {
        tr class="bg-white" hx-target="this" {
            td class="px-6 py-4 font-medium text-gray-900 whitespace-nowrap" { (username) }
            td class="px-6 py-4" { "participant" }
            td class="px-6 py-4" {
                button
                    hx-post="remove-user"
                    hx-vals={r#"{"user_id":""#(user_id)r#""}"#}
                    class="px-2 py-2 mt-1 font-bold text-white bg-orange-600 rounded hover:bg-orange-700" {
                    "Remove"
                }
            }
        }
    }
}
