use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRanking {
    pub id: i32,
    pub name: String,
    pub avatar: Option<String>,
    pub rank: i32,
    pub score: i32,
    pub correct_guesses: i32,
    pub total_guesses: i32,
}

impl PlayerRanking {
    pub fn accuracy(&self) -> f64 {
        if self.total_guesses == 0 {
            0.0
        } else {
            (self.correct_guesses as f64 / self.total_guesses as f64) * 100.0
        }
    }

    pub fn accuracy_rounded(&self) -> i32 {
        self.accuracy().round() as i32
    }

    pub fn avatar_initials(&self) -> String {
        self.name.chars().next().unwrap_or('?').to_string().to_uppercase()
    }
}

// Mock data for testing - in a real app this would come from the database
pub fn mock_player_rankings() -> Vec<PlayerRanking> {
    vec![
        PlayerRanking {
            id: 1,
            name: "Alex Johnson".to_string(),
            avatar: Some("/abstract-letter-aj.png".to_string()),
            rank: 1,
            score: 1030,
            correct_guesses: 5,
            total_guesses: 6,
        },
        PlayerRanking {
            id: 2,
            name: "Taylor Smith".to_string(),
            avatar: Some("/abstract-geometric-ts.png".to_string()),
            rank: 2,
            score: 850,
            correct_guesses: 4,
            total_guesses: 6,
        },
        PlayerRanking {
            id: 3,
            name: "Morgan Lee".to_string(),
            avatar: Some("/machine-learning-concept.png".to_string()),
            rank: 3,
            score: 820,
            correct_guesses: 4,
            total_guesses: 6,
        },
        PlayerRanking {
            id: 4,
            name: "Jordan Rivera".to_string(),
            avatar: Some("/stylized-jr-logo.png".to_string()),
            rank: 4,
            score: 550,
            correct_guesses: 2,
            total_guesses: 6,
        },
        PlayerRanking {
            id: 5,
            name: "Casey Kim".to_string(),
            avatar: Some("/abstract-ck-design.png".to_string()),
            rank: 5,
            score: 540,
            correct_guesses: 3,
            total_guesses: 6,
        },
    ]
}
