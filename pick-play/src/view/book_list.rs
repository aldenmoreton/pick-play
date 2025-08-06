use crate::model::book::BookSubscriptionStats;

pub fn markup(books: Vec<BookSubscriptionStats>) -> maud::Markup {
    maud::html! {
        div class="flex flex-col items-center justify-center" {
            ul class="items-center self-center justify-center" {
                @if books.is_empty() {
                    li class="p-3 h-30 w-60" {
                        p { "No Books Yet!" }
                    }
                }
                @for book in books {
                    li class="p-3 h-30 w-60" {
                        div class="border border-gray-300 justify-center object-fill max-w-sm overflow-hidden bg-white rounded-lg shadow-lg" {
                            a href={"/book/"(book.id)"/"} class="object-fill" {
                                h1 class="text-2xl font-bold" { (book.name) }
                                @if book.num_members > 1 {
                                    p {
                                        (book.rank)
                                        @if book.rank == 1 {
                                            "st"
                                        } @else if book.rank == 2 {
                                            "nd"
                                        } @else if book.rank == 3 {
                                            "rd"
                                        } @else if book.rank == book.num_members {
                                            span class="text-orange-950" { "Last" }
                                        } @else {
                                            "th"
                                        }
                                        " - " (book.user_points) " Points"
                                        br;
                                        span class="italic" { (book.num_members) " Members" }
                                    }
                                }
                            }
                            @if let (Some(id), Some(title), Some(is_open)) = (book.recent_chapter_id, book.recent_chapter_title, book.recent_chapter_is_open) {
                                a href={"/book/"(book.id)"/chapter/"(id)"/"} class="object-fill" {
                                    p class="bg-gray-100" {
                                        (title)
                                        br;
                                        @if is_open {
                                            span class="text-green-500" { "Open" }
                                        } @else {
                                            span class="text-red-500" { "Closed" }
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
}
