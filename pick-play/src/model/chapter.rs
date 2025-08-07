use sqlx::PgPool;

use crate::AppError;

#[derive(Debug, Clone)]
pub struct Chapter {
    pub chapter_id: i32,
    pub book_id: i32,
    pub is_open: bool,
    pub is_visible: bool,
    pub title: String,
}

pub async fn get_chapters(book_id: i32, pool: &PgPool) -> Result<Vec<Chapter>, sqlx::Error> {
    sqlx::query_as!(
        Chapter,
        r#"	SELECT id AS chapter_id, book_id, is_open, title, is_visible
			FROM chapters
			WHERE book_id = $1
            ORDER BY created_at DESC
		"#,
        book_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_chapter(chapter_id: i32, pool: &PgPool) -> Result<Chapter, sqlx::Error> {
    sqlx::query_as!(
        Chapter,
        r#"	SELECT id AS chapter_id, book_id, title, is_open, is_visible
			FROM chapters
			WHERE id = $1
		"#,
        chapter_id
    )
    .fetch_one(pool)
    .await
}

pub struct ChapterUser {
    pub user_id: i32,
    pub username: String,
    pub total_points: i32,
    pub rank: i32,
}

pub async fn get_chapter_users(
    book_id: i32,
    chapter_id: i32,
    pool: &PgPool,
) -> Result<Vec<ChapterUser>, AppError> {
    sqlx::query_as!(
        ChapterUser,
        r#"
        SELECT
            user_id,
            username,
            COALESCE(total_points, 0)::INT as "total_points!",
            RANK() OVER (ORDER BY total_points DESC, username)::INT as "rank!"
        FROM (
            SELECT
                sub1.id AS user_id,
                sub1.USERNAME,
                SUM(COALESCE(sub2.POINTS, 0)) AS TOTAL_POINTS
            FROM (
                SELECT users.id, users.username
                FROM users
                JOIN subscriptions on users.id = subscriptions.user_id
                WHERE book_id = $1 AND COALESCE(((subscriptions.role->'guest'->'chapter_ids') @> to_jsonb($2::INT)), true)
            ) as sub1
            LEFT JOIN (
                SELECT picks.user_id, picks.points
                FROM picks
                WHERE picks.chapter_id = $2
            ) as sub2 on sub1.id = sub2.user_id
            GROUP BY
                sub1.ID,
                sub1.USERNAME
        ) AS sub3
        ORDER BY total_points DESC, username
        "#,
        book_id,
        chapter_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub struct ChapterStats {
    pub id: i32,
    pub title: String,
    pub total_points: i32,
    pub user_points: i32,
    pub user_rank: i32,
    pub is_open: bool,
    pub is_visible: bool,
}

pub async fn chapters_with_stats(
    user_id: i32,
    book_id: i32,
    pool: &PgPool,
) -> Result<Vec<ChapterStats>, sqlx::Error> {
    sqlx::query_as!(
        ChapterStats,
        r#"
        SELECT
            c.id,
            c.title,
            c.is_open,
            c.is_visible,
            COALESCE((
                SELECT
                    COALESCE(SUM(CASE
                        WHEN event_type = 'spread_group' THEN (SELECT SUM(num) FROM generate_series(1, JSONB_ARRAY_LENGTH(contents->'spread_group')) AS num)
                        WHEN event_type = 'user_input' THEN (contents->'user_input'->>'points')::INT
                        ELSE 0
                    END), 0)
                FROM events
                WHERE events.chapter_id = c.id
            )::INT, 0) AS "total_points!",
            COALESCE((
                SELECT COALESCE(SUM(points)::INT, 0)
                FROM picks
                WHERE user_id = $1 AND chapter_id = c.id
            ), 0) AS "user_points!",
            COALESCE((
                SELECT COALESCE(rank, 0)::INT
                FROM (
                    SELECT user_id, RANK() OVER (ORDER BY SUM(points) DESC) as rank
                    FROM picks
                    WHERE chapter_id = c.id
                    GROUP BY user_id
                ) ranked_users
                WHERE user_id = $1
            ), 1) AS "user_rank!"
        FROM chapters AS c
        WHERE book_id = $2
        ORDER BY c.created_at DESC
    "#,
        user_id,
        book_id
    )
    .fetch_all(pool)
    .await
}

// pub struct ChapterLeaderboardStats {
//     pub user_id: i32,
//     pub username: String,
//     pub user_points: i32,
//     pub user_rank: i32,
// }

// pub async fn chapter_with_stats(
//     chapter_id: i32,
//     pool: &PgPool,
// ) -> Result<Vec<ChapterLeaderboardStats>, sqlx::Error> {
//     todo!()
// }
