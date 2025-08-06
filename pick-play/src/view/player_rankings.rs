use maud::{html, Markup};
use crate::model::player_ranking::PlayerRanking;

pub fn player_rankings_card(rankings: Vec<PlayerRanking>) -> Markup {
    html! {
        // Card Container
        div class="bg-white rounded-lg border border-gray-200 shadow-sm overflow-hidden" {
            // Card Header
            div class="px-6 py-4 border-b border-gray-100" {
                h3 class="text-lg font-semibold text-gray-900" { "Player Rankings" }
                p class="text-sm text-gray-600 mt-1" { "Overall performance across all questions" }
            }
            
            // Card Content
            div class="p-0" {
                // Table Container with rounded border and overflow
                div class="border border-gray-200 rounded-lg overflow-hidden" {
                    div class="overflow-x-auto" {
                        table class="w-full" {
                            // Table Header
                            thead class="bg-gray-50" {
                                tr {
                                    th class="w-20 px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider" { "Rank" }
                                    th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Player" }
                                    th class="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider" { "Correct" }
                                    th class="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider" { "Accuracy" }
                                    th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider" { "Total Score" }
                                }
                            }
                            
                            // Table Body
                            tbody class="bg-white divide-y divide-gray-200" {
                                @for player in rankings.iter() {
                                    tr class="hover:bg-gray-50 transition-colors duration-150" {
                                        // Rank Column
                                        td class="px-4 py-4 text-center text-sm font-medium text-gray-900" { 
                                            (player.rank) 
                                        }
                                        
                                        // Player Column with Avatar
                                        td class="px-4 py-4" {
                                            div class="flex items-center space-x-3" {
                                                // Avatar Container
                                                div class="flex-shrink-0" {
                                                    @if let Some(avatar_url) = &player.avatar {
                                                        img class="h-8 w-8 rounded-full object-cover" src=(avatar_url) alt=(player.name);
                                                    } @else {
                                                        // Fallback Avatar with Initials
                                                        div class="h-8 w-8 rounded-full bg-gray-200 flex items-center justify-center" {
                                                            span class="text-xs font-medium text-gray-600" { (player.avatar_initials()) }
                                                        }
                                                    }
                                                }
                                                // Player Name
                                                div class="flex-1 min-w-0" {
                                                    p class="text-sm font-medium text-gray-900 truncate" { (player.name) }
                                                }
                                            }
                                        }
                                        
                                        // Correct Answers Column
                                        td class="px-4 py-4 text-center text-sm text-gray-900" { 
                                            (player.correct_guesses) " / " (player.total_guesses)
                                        }
                                        
                                        // Accuracy Column with Percentage Styling
                                        td class="px-4 py-4 text-center" {
                                            @let accuracy = player.accuracy_rounded();
                                            @let color_class = if accuracy >= 80 {
                                                "text-green-600 bg-green-100"
                                            } else if accuracy >= 60 {
                                                "text-yellow-600 bg-yellow-100"
                                            } else {
                                                "text-red-600 bg-red-100"
                                            };
                                            span class={ "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " (color_class) } {
                                                (accuracy) "%"
                                            }
                                        }
                                        
                                        // Total Score Column
                                        td class="px-4 py-4 text-right text-sm font-bold text-lg text-gray-900" { 
                                            (player.score)
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

