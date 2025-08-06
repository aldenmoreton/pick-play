use std::iter::Peekable;

use crate::model::chapter::ChapterStats;

pub fn m<'a, I>(
    book_id: i32,
    mut chapters: Peekable<I>,
    empty_message: Option<&str>,
) -> maud::Markup
where
    I: Iterator<Item = &'a ChapterStats>,
{
    maud::html! {
        div class="flex flex-col items-center justify-center" {
            ul class="items-center self-center justify-center" {
                @if chapters.peek().is_none() {
                    li class="p-3 h-30 w-60" {
                        p { (empty_message.unwrap_or("No chapters")) }
                    }
                }
                @for chapter in chapters {
                    li {
                        a href={"/book/"(book_id)"/chapter/"(chapter.id)"/"} class="object-fill" {
                            div class="border border-gray-300 justify-center p-3 m-3 bg-white rounded-lg shadow-lg h-30 w-60" {
                                p { (chapter.title) }
                                p {
                                    @if chapter.is_open {
                                        span class="text-green-500" { "Open" }
                                    } @else {
                                        "1st Place"
                                        br;
                                        (chapter.user_points) "/" (chapter.total_points) " Points"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
