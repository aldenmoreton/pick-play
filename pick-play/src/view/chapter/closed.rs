use std::collections::HashMap;

use crate::{
    controllers::auth::BackendUser,
    model::{
        book::{BookRole, BookSubscription},
        chapter::{Chapter, ChapterUser},
        event::{ChapterPick, ChapterPickHash, Event, EventContent},
    },
};

pub fn m(
    curr_user: BackendUser,
    chapter: &Chapter,
    book_subscription: &BookSubscription,
    users: &[ChapterUser],
    user_picks: &HashMap<ChapterPickHash, ChapterPick>,
    events: &[Event],
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    crate::view::authenticated(
        &curr_user.username,
        None,
        None,
        Some(maud::html!(
            link rel="stylesheet" id="tailwind" href="/public/styles/chapter-table.css";
            style {
                (maud::PreEscaped(r#"
                .toggle-pill {
                    position: relative;
                    background: #f3f4f6;
                    padding: 4px;
                    border-radius: 9999px;
                    display: flex;
                    box-shadow: inset 0 2px 4px 0 rgba(0, 0, 0, 0.1);
                }

                .toggle-slider {
                    position: absolute;
                    top: 4px;
                    left: 4px;
                    height: calc(100% - 8px);
                    background: linear-gradient(135deg, #3b82f6, #1d4ed8);
                    border-radius: 9999px;
                    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
                    box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
                    z-index: 1;
                }

                .toggle-button {
                    position: relative;
                    z-index: 2;
                    flex: 1;
                    padding: 8px 16px;
                    text-align: center;
                    font-size: 14px;
                    font-weight: 500;
                    border-radius: 9999px;
                    transition: color 0.3s ease;
                    cursor: pointer;
                    border: none;
                    background: transparent;
                    color: #6b7280;
                }

                .toggle-button.active {
                    color: white;
                }

                .toggle-button:not(.active):hover {
                    color: #374151;
                }

                .leaderboard-table {
                    max-height: calc(100vh - 120px);
                }

                /* Mobile section visibility */
                @media (max-width: 767px) {
                    .section-content {
                        display: none;
                    }
                    .section-content.active {
                        display: block;
                    }
                }

                @media (min-width: 768px) {
                    .mobile-toggle-container {
                        display: none !important;
                    }
                    .leaderboard-table {
                        max-height: 24rem;
                    }
                    /* Show all sections on desktop */
                    .section-content {
                        display: block !important;
                    }
                }
                "#))
            }
            script {
                (maud::PreEscaped(r#"
                function isMobileScreen() {
                    return window.innerWidth < 768; // md breakpoint
                }

                let currentActiveSection = 'leaderboard'; // Track the current active section
                let isInitialized = false; // Track if we've initialized

                window.showSection = function(section) {
                    // Update the current active section
                    currentActiveSection = section;

                    // Only run toggle functionality on mobile screens
                    if (!isMobileScreen()) {
                        return;
                    }

                    // Update section visibility using classes
                    const sections = ['leaderboard', 'events', 'table'];
                    sections.forEach(s => {
                        const el = document.getElementById(s + '-section');
                        if (el) {
                            if (s === section) {
                                el.classList.add('active');
                            } else {
                                el.classList.remove('active');
                            }
                        }
                    });

                    // Update button states
                    document.querySelectorAll('.toggle-button').forEach(btn => {
                        btn.classList.remove('active');
                    });

                    const activeBtn = document.getElementById(section + '-btn');
                    if (activeBtn) {
                        activeBtn.classList.add('active');
                    }

                    // Move slider
                    const slider = document.querySelector('.toggle-slider');
                    const buttons = document.querySelectorAll('.toggle-button');
                    const activeIndex = Array.from(buttons).findIndex(btn => btn.id === section + '-btn');

                    if (slider && activeIndex !== -1) {
                        const buttonWidth = 100 / buttons.length;
                        slider.style.width = buttonWidth + '%';
                        slider.style.transform = 'translateX(' + (activeIndex * 100) + '%)';
                    }
                };

                function handleResize() {
                    if (isMobileScreen()) {
                        // Always call showSection to ensure proper button and slider state
                        showSection(currentActiveSection);
                        if (!isInitialized) {
                            isInitialized = true;
                        }
                    } else {
                        // On desktop, the CSS handles showing all sections
                        isInitialized = true;
                    }
                }

                // Initialize on page load
                document.addEventListener('DOMContentLoaded', function() {
                    // Run immediately to prevent flicker
                    handleResize();
                });

                // Handle window resize
                window.addEventListener('resize', handleResize);
                "#))
            }
        )),
        Some(maud::html! {
            p {
                a href="/" class="text-blue-400 hover:underline" {"Home"} " > "
                a href="../.." class="text-blue-400 hover:underline" { (book_subscription.name) } " > "
                a {(chapter.title)}
            }
        }),
        Some(maud::html! {
            div class="flex flex-col flex-grow min-h-screen bg-gray-50" {
                @if book_subscription.role == BookRole::Admin {
                    a href="admin/" {
                        button class="fixed z-50 px-3 py-2 text-sm font-bold text-white transition-colors bg-orange-600 rounded-full shadow-lg bottom-4 right-4 hover:bg-orange-700" {
                            "Admin"
                        }
                    }
                }
                div class="mx-4 mb-3 mobile-toggle-container" {
                    div class="max-w-sm mx-auto toggle-pill" {
                        div class="toggle-slider" {}
                        button
                            id="leaderboard-btn"
                            class="toggle-button active"
                            onclick="showSection('leaderboard')" {
                            "Leaderboard"
                        }
                        button
                            id="events-btn"
                            class="toggle-button"
                            onclick="showSection('events')" {
                            "Events"
                        }
                        button
                            id="table-btn"
                            class="toggle-button"
                            onclick="showSection('table')" {
                            "Table"
                        }
                    }
                }

                div id="leaderboard-section" class="section-content active mx-4" {
                    (leaderboard(&chapter.title, users, events, user_picks))
                }

                div id="events-section" class="section-content mx-4" {
                    h2 class="hidden mb-4 text-xl font-bold text-gray-900 md:block" { "Event Results" }
                    (event_tiles(events, users, user_picks, relevent_teams))
                }

                div id="table-section" class="section-content mx-4" {
                    div class="overflow-hidden md:bg-white md:border md:border-gray-200 md:rounded-lg md:shadow-md" {
                        div class="hidden p-4 bg-gray-100 border-b md:block" {
                            h2 class="text-xl font-bold text-gray-900" { "Detailed Results Table" }
                        }
                        div class="overflow-x-auto" {
                            table class="w-full picktable" {
                                (table_header(events, relevent_teams))
                                (table_rows(events, users, user_picks, relevent_teams))
                            }
                        }
                    }
                }
            }
        }),
        None,
    )
}

fn user_points(
    user: &ChapterUser,
    events: &[Event],
    user_picks: &HashMap<ChapterPickHash, ChapterPick>,
) -> (i32, i32) {
    let mut correct = 0;
    let mut total = 0;

    for event in events {
        let user_pick = user_picks.get(&ChapterPickHash {
            event_id: event.id,
            user_id: user.user_id,
        });
        match (&event.contents.0, &user_pick) {
            (EventContent::SpreadGroup(spreads), Some(ChapterPick::SpreadGroup { choice, .. })) => {
                correct += spreads
                    .iter()
                    .zip(choice)
                    .filter(|(spread, choice)| matches!(spread.answer.clone(), Some(a) if a == **choice))
                    .count() as i32;
                total += spreads.len() as i32;
            }
            (EventContent::SpreadGroup(spreads), None) => {
                total += spreads.len() as i32;
            }
            (EventContent::UserInput(_), None) => total += 1 as i32,
            (EventContent::UserInput(input), Some(ChapterPick::UserInput { choice, .. })) => {
                correct += input
                    .acceptable_answers
                    .as_ref()
                    .map(|answers| answers.contains(choice))
                    .unwrap_or_default() as i32;
                total += 1;
            }
            _ => (),
        }
    }

    (correct, total)
}

fn leaderboard(
    title: &str,
    users: &[ChapterUser],
    events: &[Event],
    user_picks: &HashMap<ChapterPickHash, ChapterPick>,
) -> maud::Markup {
    maud::html!(
        div class="md:bg-white md:border md:border-gray-300 md:shadow-lg md:rounded-xl" {
            div class="hidden p-6 pb-4 text-left bg-gray-500 border-b rounded-t-xl md:block" {
                h1 class="text-2xl font-bold text-white" { "Leaderboard" br; (title) }
            }
            div class="md:p-6" {
                div class="overflow-hidden border border-gray-300 rounded-lg shadow-lg bg-gray-50" {
                    div class="overflow-y-auto leaderboard-table" {
                        table class="w-full" {
                            thead class="sticky top-0 bg-white border-b shadow-sm" {
                                tr {
                                    th class="w-20 px-3 py-2 text-sm font-medium text-center text-gray-900" { "Rank" }
                                    th class="px-3 py-2 text-sm font-medium text-left text-gray-900" { "Player" }
                                    th class="px-3 py-2 text-sm font-medium text-center text-gray-900" { "Correct" }
                                    th class="px-3 py-2 text-sm font-medium text-right text-gray-900" { "Points" }
                                }
                            }
                            tbody class="bg-white divide-y divide-gray-200" {
                                @for user in users {
                                    tr class="hover:bg-gray-50" {
                                        td class="px-3 py-2 font-medium text-center text-gray-900" { (user.rank) }
                                        td class="px-3 py-2" {
                                            div class="flex items-center gap-2" {
                                                span class="font-medium text-gray-900" { (user.username) }
                                            }
                                        }
                                        @let correct_questions = user_points(user, events, user_picks);
                                        td class="px-3 py-2 text-center text-gray-900" { (correct_questions.0) " / " (correct_questions.1) }
                                        td class="px-3 py-2 font-bold text-right text-gray-900" { (user.total_points) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}

fn event_tiles(
    events: &[Event],
    users: &[ChapterUser],
    user_picks: &HashMap<ChapterPickHash, ChapterPick>,
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    maud::html!(
        div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3" {
            @for event in events {
                (event_tile(event, users, user_picks, relevent_teams))
            }
        }
    )
}

fn event_tile(
    event: &Event,
    users: &[ChapterUser],
    user_picks: &HashMap<ChapterPickHash, ChapterPick>,
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    match &event.contents.0 {
        EventContent::SpreadGroup(spreads) => maud::html!(
            @for (i, spread) in spreads.iter().enumerate() {
                (spread_tile(i, spread, event, users, user_picks, relevent_teams))
            }
        ),
        EventContent::UserInput(input) => user_input_tile(input, event, users, user_picks),
    }
}

fn user_input_tile(
    input: &crate::model::user_input::UserInput,
    event: &Event,
    users: &[ChapterUser],
    user_picks: &HashMap<ChapterPickHash, ChapterPick>,
) -> maud::Markup {
    maud::html!(
        div class="bg-white border border-gray-300 rounded-lg shadow-md" {
            div class="p-4 pb-2" {
                div class="flex items-start justify-between mb-2" {
                    div class="flex-1 mr-4 text-left" {
                        h3 class="mb-1 text-lg font-semibold text-left text-gray-900" { (input.title) }
                        @if let Some(desc) = &input.description {
                            p class="text-sm text-left text-gray-600" { (desc) }
                        }
                    }
                    div class="flex-shrink-0 text-right" {
                        span class="text-xl font-bold text-blue-600" { (input.points) }
                        p class="text-sm text-gray-500" { "Point" @if input.points > 1 {"s"} }
                    }
                }
            }
            div class="p-4 pt-0" {
                div class="space-y-2" {
                    div class="space-y-2 overflow-y-auto max-h-48 overscroll-contain" {
                        @for user in users {
                            @let user_pick = user_picks.get(&ChapterPickHash{event_id: event.id, user_id: user.user_id});
                            @match user_pick {
                                Some(ChapterPick::UserInput{choice, wager: _wager, points}) => {
                                    @let (bg_color, icon) = match points {
                                        Some(0) => ("bg-red-50 border-red-200", "✗"),
                                        Some(_) => ("bg-green-50 border-green-200", "✓"),
                                        None => ("bg-gray-50", "?")
                                    };
                                    div class=(format!("border flex items-center justify-between p-2 rounded-md {}", bg_color)) {
                                        div class="flex items-center gap-2" {
                                            span class="font-medium text-gray-900" { (user.username) }
                                        }
                                        div class="text-right" {
                                            div class="flex items-center gap-1" {
                                                span class="text-sm text-gray-700 truncate max-w-24" title={(choice)} { (choice) }
                                                span class="text-sm" { (icon) }
                                            }
                                        }
                                    }
                                },
                                _ => div class="flex items-center justify-between p-2 border rounded-md bg-gray-50" {
                                    div class="flex items-center gap-2" {
                                        span class="font-medium text-gray-900" { (user.username) }
                                    }
                                    div class="text-right" {
                                        div class="flex items-center gap-1" {
                                            span class="text-sm text-gray-700 truncate max-w-24" { "No Pick" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}

fn spread_tile(
    index: usize,
    spread: &crate::model::spread::Spread,
    event: &Event,
    users: &[ChapterUser],
    user_picks: &HashMap<ChapterPickHash, ChapterPick>,
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    let mut points_wagered = 0;
    let mut points_awarded = 0;
    for user in users {
        if let Some(ChapterPick::SpreadGroup { choice, wager, .. }) =
            user_picks.get(&ChapterPickHash {
                event_id: event.id,
                user_id: user.user_id,
            })
        {
            points_wagered += wager[index];
            match &event.contents.0 {
                EventContent::SpreadGroup(spreads)
                    if spreads[index]
                        .answer
                        .as_ref()
                        .map(|ans| *ans == choice[index])
                        .unwrap_or_default() =>
                {
                    points_awarded += wager[index]
                }
                _ => (),
            }
        }
    }

    let team_win = |team| {
        spread
            .answer
            .as_ref()
            .map(|ans| ans == team)
            .unwrap_or_default()
    };
    let is_answered = spread
        .answer
        .as_ref()
        .map(|ans| ans != "unpicked")
        .unwrap_or_default();

    maud::html!(
        div class="bg-white border border-gray-300 rounded-lg shadow-md" {
            div class="p-4 pb-2" {
                div class="flex items-center justify-between mb-3" {
                    div class="text-left" {
                        h3.text-red-500[is_answered] class="text-base font-semibold" {
                            span.text-green-500[team_win("away")] { (relevent_teams[&spread.away_id].0) }
                            span class="text-sm font-normal text-gray-500" { (format!(" ({:+})", -1. * spread.home_spread)) }
                            span class="ml-2 text-sm font-normal text-gray-500" { "at" }
                            br;
                            span.text-green-500[team_win("home")]{ (relevent_teams[&spread.home_id].0) }
                            span class="text-sm font-normal text-gray-500" { (format!(" ({:+})", spread.home_spread)) }
                        }
                    }
                    div class="text-right" {
                        p class="text-sm text-gray-600" { "Wagered: " (points_wagered) }
                        p class="text-sm text-gray-600" { "Awarded: " (points_awarded) }
                    }
                }
            }
            div class="p-4 pt-0" {
                div class="space-y-2" {
                    div class="space-y-2 overflow-y-auto max-h-48 overscroll-contain" {
                        @for user in users {
                            @let user_pick = user_picks.get(&ChapterPickHash{event_id: event.id, user_id: user.user_id});
                            @match user_pick {
                                Some(ChapterPick::SpreadGroup{choice, wager, ..}) => {
                                    @let is_correct = spread.answer.as_ref().map(|a| *a == choice[index]).unwrap_or(false);
                                    @let bg_color = if !is_answered {
                                        "bg-gray-50"
                                    } else if is_correct {
                                        "bg-green-50 border-green-200"
                                    } else {
                                        "bg-red-50 border-red-200"
                                    };

                                    @let team_id = match choice[index].as_str() {
                                        "home" => spread.home_id,
                                        "away" => spread.away_id,
                                        _ => panic!()
                                    };

                                    div class={(format!("flex items-center justify-between p-2 rounded-md border {}", bg_color))} {
                                        div class="flex items-center gap-2" {
                                            span class="font-medium text-gray-900" { (user.username) }
                                        }
                                        div class="text-right" {
                                            div class="flex items-center gap-1" {
                                                div class="text-right" {
                                                    p class="text-sm font-medium text-gray-900" { (relevent_teams[&team_id].0) }
                                                    p class="text-xs text-gray-500" { "Wager: " (wager[index]) }
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => div class="flex items-center justify-between p-2 rounded-md border bg-gray-50{}" {
                                    div class="flex items-center gap-2" {
                                        span class="font-medium text-gray-900" { (user.username) }
                                    }
                                    div class="text-right" {
                                        div class="flex items-center gap-1" {
                                            div class="text-right" {
                                                p class="text-sm font-medium text-gray-900" { "No Pick" }
                                                p class="text-xs text-gray-500" { "Wager: 0" }
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
    )
}

fn table_header(
    events: &[Event],
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    maud::html!(
        thead class="sticky top-0 bg-gray-50" {
            tr {
                th class="px-4 py-3 text-sm font-medium text-left text-gray-700 border-b border-gray-200" { "Player" }
                @for event in events {
                    @match &event.contents.0 {
                        EventContent::SpreadGroup(group) => {
                            @for spread in group {
                                th class="px-3 py-3 text-sm font-medium text-center text-gray-700 border-b border-gray-200 min-w-32" {
                                    div class="space-y-1" {
                                        p class="text-xs" { (relevent_teams[&spread.away_id].0) }
                                        p class="text-xs text-gray-500" { (format!("({:+})", -1. * spread.home_spread)) }
                                        p class="text-xs" { "at" }
                                        p class="text-xs" { (relevent_teams[&spread.home_id].0) }
                                    }
                                }
                            }
                        },
                        EventContent::UserInput(input) => {
                            th class="px-3 py-3 text-sm font-medium text-center text-gray-700 border-b border-gray-200 min-w-24" {
                                p class="text-xs" { (input.title) }
                            }
                        }
                    }
                }
            }
        }
    )
}

fn table_rows(
    events: &[Event],
    users: &[ChapterUser],
    picks_by_user: &HashMap<ChapterPickHash, ChapterPick>,
    relevent_teams: &HashMap<i32, (String, Option<String>)>,
) -> maud::Markup {
    maud::html!(
        tbody class="divide-y divide-gray-200" {
            // Each user
            @for ChapterUser { user_id, username, total_points, rank: _rank } in users {
                tr class="hover:bg-gray-50" {
                    td class="px-4 py-3 border-b border-gray-200 bg-gray-200 opacity-100" {
                        p class="font-medium text-gray-900" {(username)}
                        p class="text-sm text-gray-500" {(total_points) " point" (if *total_points != 1 {"s"} else {""})}
                    }
                    // Each event
                    @for event in events {
                        // Event type
                        @match (&event.contents.0, picks_by_user.get(&ChapterPickHash{event_id: event.id, user_id: *user_id})) {
                            (EventContent::SpreadGroup(spreads), Some(ChapterPick::SpreadGroup { choice, wager, .. })) => {
                                @for (i, spread) in spreads.iter().enumerate() {
                                    @let bg_color = match spread.answer.as_ref().map(|a| *a == choice[i]) {
                                        _ if spread.answer.as_ref().map(|a| *a == "push").unwrap_or(false) => "bg-orange-100 text-orange-800",
                                        _ if spread.answer.as_ref().map(|a| *a == "unpicked").unwrap_or(false) => "bg-gray-50",
                                        Some(true) => "bg-green-100 text-green-800",
                                        Some(false) => "bg-red-100 text-red-800",
                                        None => "bg-gray-100"
                                    };

                                    @let team_id = match choice[i].as_str() {
                                        "home" => spread.home_id,
                                        "away" => spread.away_id,
                                        _ => panic!()
                                    };

                                    td class={(format!("px-3 py-3 text-center border-b border-gray-200 {}", bg_color))} {
                                        div class="space-y-1" {
                                            p class="text-xs font-medium" {(relevent_teams[&team_id].0)}
                                            p class="text-xs opacity-75" {"Wager: " (wager[i])}
                                        }
                                    }
                                }
                            },
                            (EventContent::SpreadGroup(spreads), None) => {
                                @for _ in spreads {
                                    td class="px-3 py-3 text-center border-b border-gray-50 bg-gray-50" {
                                        p class="text-xs font-medium text-red-600" {"No Pick"}
                                    }
                                }
                            },
                            (EventContent::UserInput(_), Some(ChapterPick::UserInput { choice, wager, points })) => {
                                @let bg_color = match points.as_ref().map(|p| p == wager) {
                                    Some(true) => "bg-green-100 text-green-800",
                                    Some(false) => "bg-red-100 text-red-800",
                                    None => "bg-gray-100"
                                };

                                td class={(format!("px-3 py-3 text-center border-b {}", bg_color))} {
                                    div class="space-y-1" {
                                        p class="text-xs font-medium truncate" title={(choice)} {(choice)}
                                        p class="text-xs opacity-75" {"Wager: " (wager)}
                                    }
                                }
                            }
                            (EventContent::UserInput(_), None) => {
                                td class="px-3 py-3 text-center border-b bg-gray-50 border-gray-50" {
                                    p class="text-xs font-medium text-red-600" {"No Pick"}
                                }
                            }
                            _ => {
                                td class="px-3 py-3 text-center border-b border-gray-200 bg-yellow-50" {
                                    p class="text-xs text-yellow-800" { "Error" }
                                }
                            }
                        }

                    }
                }
            }
        }
    )
}
