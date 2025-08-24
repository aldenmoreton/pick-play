use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookRole {
    Owner,
    Admin,
    Participant,
    Guest { chapter_ids: Vec<i32> },
    Unauthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookSubscription {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    #[sqlx(json)]
    pub role: BookRole,
}

pub async fn get_books(user_id: i32, pool: &PgPool) -> Result<Vec<BookSubscription>, AppError> {
    let result = sqlx::query_as::<_, BookSubscription>(
        r#"	SELECT b.id AS id, b.name, s.role, s.user_id
			FROM books AS b
			INNER JOIN subscriptions AS s ON s.book_id=b.id
			WHERE s.user_id = $1
		"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn get_book(
    user_id: i32,
    book_id: i32,
    pool: &PgPool,
) -> Result<BookSubscription, sqlx::Error> {
    sqlx::query_as::<_, BookSubscription>(
        r#"
            SELECT b.id AS id, b.name, s.role, s.user_id
            FROM books AS b
            INNER JOIN subscriptions AS s ON s.book_id=b.id
            WHERE s.user_id = $1 AND b.id = $2
            "#,
    )
    .bind(user_id)
    .bind(book_id)
    .fetch_one(pool)
    .await
}

pub async fn get_book_users(book_id: i32, pool: &PgPool) -> Result<Box<[(i32, String)]>, AppError> {
    Ok(sqlx::query!(
        "
            SELECT users.id, users.username
            FROM users
            JOIN subscriptions ON users.id = subscriptions.user_id
            WHERE subscriptions.book_id = $1
            ",
        book_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?
    .into_iter()
    .map(|r| (r.id, r.username))
    .collect())
}

#[derive(Debug, sqlx::FromRow)]
pub struct BookSubscriptionStats {
    pub id: i32,
    pub name: String,
    pub num_members: i32,
    pub rank: i32,
    pub user_points: i32,
    pub recent_chapter_id: Option<i32>,
    pub recent_chapter_title: Option<String>,
    pub recent_chapter_is_open: Option<bool>,
}

pub async fn user_books_stats(
    user_id: i32,
    pool: &PgPool,
) -> Result<Vec<BookSubscriptionStats>, sqlx::Error> {
    sqlx::query_as!(
        BookSubscriptionStats,
        r#"
        WITH user_book_stats AS (
            SELECT
                book_id,
                user_id,
                -- Calculate total points from picks/events
                COALESCE((
                    SELECT SUM(p.points)
                    FROM picks p
                    WHERE p.book_id = s.book_id AND p.user_id = s.user_id
                ), 0) +
                -- Calculate total extra points
                COALESCE((
                    SELECT SUM(ap.points)
                    FROM added_points ap
                    WHERE ap.book_id = s.book_id AND ap.user_id = s.user_id
                ), 0) AS total_points
            FROM subscriptions s
        ),
        user_rankings AS (
            SELECT
                book_id,
                user_id,
                total_points,
                RANK() OVER (PARTITION BY book_id ORDER BY total_points DESC) as user_rank
            FROM user_book_stats
        )
        SELECT
            b.id AS "id!",
            b.name AS "name!",
            (SELECT COUNT(*) FROM subscriptions WHERE book_id = b.id AND not role ? 'guest')::INT AS "num_members!",
            (SELECT c.id FROM chapters AS c WHERE c.book_id = b.id AND c.is_visible ORDER BY c.created_at DESC LIMIT 1) AS recent_chapter_id,
            (SELECT c.title FROM chapters AS c WHERE c.book_id = b.id AND c.is_visible ORDER BY c.created_at DESC LIMIT 1) AS recent_chapter_title,
            (SELECT c.is_open FROM chapters AS c WHERE c.book_id = b.id AND c.is_visible ORDER BY c.created_at DESC LIMIT 1) AS recent_chapter_is_open,
            ur.total_points::INT AS "user_points!",
            ur.user_rank::INT AS "rank!"
        FROM subscriptions AS s
        JOIN books AS b ON s.book_id = b.id
        LEFT JOIN user_rankings ur ON ur.book_id = b.id AND ur.user_id = s.user_id
        WHERE s.user_id = $1
        ORDER BY b.created_at DESC;
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BookRanking {
    pub user_id: i32,
    pub username: String,
    pub points: i32,
    pub rank: i32,
}

pub async fn book_rank(
    user_id: i32,
    book_id: i32,
    pool: &PgPool,
) -> Result<BookRanking, sqlx::Error> {
    sqlx::query_as!(
        BookRanking,
        r#"
        WITH user_event_points AS (
          -- Points from picks/events
          SELECT
            p.user_id,
            p.book_id,
            COALESCE(SUM(p.points), 0) AS event_points
          FROM picks p
          WHERE p.book_id = $2  -- Replace $1 with the specific book_id
          GROUP BY p.user_id, p.book_id
        ),
        user_added_points AS (
          -- Extra/added points
          SELECT
            ap.user_id,
            ap.book_id,
            COALESCE(SUM(ap.points), 0) AS extra_points
          FROM added_points ap
          WHERE ap.book_id = $2  -- Replace $1 with the specific book_id
          GROUP BY ap.user_id, ap.book_id
        ),
        user_rankings AS (
          -- Calculate rankings for ALL users first
          SELECT
            s.user_id,
            s.book_id,
            u.username,
            COALESCE(uep.event_points, 0) + COALESCE(uap.extra_points, 0) AS total_points,
            RANK() OVER (ORDER BY (COALESCE(uep.event_points, 0) + COALESCE(uap.extra_points, 0)) DESC) as ranking
          FROM subscriptions s
          JOIN users u ON s.user_id = u.id
          LEFT JOIN user_event_points uep ON s.user_id = uep.user_id AND s.book_id = uep.book_id
          LEFT JOIN user_added_points uap ON s.user_id = uap.user_id AND s.book_id = uap.book_id
          WHERE s.book_id = $2  -- Replace $1 with the specific book_id
        )
        -- Now filter to show only the specific user's ranking
        SELECT
          user_id,
          username,
          total_points::INT AS "points!",
          ranking::INT AS "rank!"
        FROM user_rankings
        WHERE user_id = $1  -- Replace $2 with the specific user_id
        ORDER BY ranking;
        "#,
        user_id,
        book_id
    )
    .fetch_one(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BookRankingStats {
    pub user_id: i32,
    pub username: String,
    pub earned_points: i32,
    pub added_points: i32,
    pub total_points: i32,
    pub rank: i32,
}

pub async fn leaderboard(
    book_id: i32,
    pool: &PgPool,
) -> Result<Vec<BookRankingStats>, sqlx::Error> {
    sqlx::query_as!(
        BookRankingStats,
        r#"
        WITH earned_points AS (
            SELECT
                user_id,
                COALESCE(SUM(points), 0) AS points
            FROM picks
            WHERE book_id = $1
            GROUP BY user_id
        ),
        added_points AS (
            SELECT
                user_id,
                COALESCE(SUM(points), 0) AS points
            FROM added_points
            WHERE book_id = $1
            GROUP BY user_id
        ),
        regular_users AS (
            SELECT
                users.id AS user_id,
                users.username,
                COALESCE(earned_points.points, 0) AS earned_points,
                COALESCE(added_points.points, 0) AS added_points,
                COALESCE(earned_points.points, 0) + COALESCE(added_points.points, 0) AS total_points
            FROM subscriptions
            JOIN users ON subscriptions.user_id = users.id
            LEFT JOIN earned_points ON users.id = earned_points.user_id
            LEFT JOIN added_points ON users.id = added_points.user_id
            WHERE subscriptions.book_id = $1 AND NOT (subscriptions.role ? 'guest')
        ),
        guest_users AS (
            SELECT
                -1 AS user_id,  -- Special ID for guests group
                'Guests' AS username,
                COALESCE(SUM(earned_points.points), 0) AS earned_points,
                COALESCE(SUM(added_points.points), 0) AS added_points,
                COALESCE(SUM(earned_points.points), 0) + COALESCE(SUM(added_points.points), 0) AS total_points
            FROM subscriptions
            JOIN users ON subscriptions.user_id = users.id
            LEFT JOIN earned_points ON users.id = earned_points.user_id
            LEFT JOIN added_points ON users.id = added_points.user_id
            WHERE subscriptions.book_id = $1 AND (subscriptions.role ? 'guest')
        ),
        combined AS (
            SELECT * FROM regular_users
            UNION ALL
            SELECT * FROM guest_users
            WHERE total_points > 0  -- Only include guests if they have points
        )
        SELECT
            user_id AS "user_id!",
            username AS "username!",
            earned_points::INT AS "earned_points!",
            added_points::INT AS "added_points!",
            total_points::INT AS "total_points!",
            RANK() OVER (ORDER BY total_points DESC)::INT AS "rank!"
        FROM combined
        ORDER BY total_points DESC;
        "#,
        book_id
    ).fetch_all(pool).await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BookMember {
    pub id: i32,
    pub username: String,
    pub role: serde_json::Value,
}

pub async fn get_book_members(
    book_id: i32,
    owner_user_id: i32,
    pool: &PgPool,
) -> Result<Vec<BookMember>, sqlx::Error> {
    sqlx::query_as!(
        BookMember,
        r#"
        SELECT u.id, u.username, s.role
        FROM users AS u
        JOIN subscriptions AS s ON u.id=s.user_id
        JOIN books AS b on s.book_id=b.id
        WHERE b.id = $1 AND u.id != $2
        ORDER BY u.id
        "#,
        book_id,
        owner_user_id
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserSearchResult {
    pub id: i32,
    pub username: String,
}

pub async fn search_users_not_in_book(
    search_username: &str,
    book_id: i32,
    pool: &PgPool,
) -> Result<Vec<UserSearchResult>, sqlx::Error> {
    sqlx::query_as!(
        UserSearchResult,
        r#"
        SELECT u.id, u.username
        FROM users AS u
        LEFT JOIN (
            SELECT *
            FROM subscriptions
            WHERE subscriptions.book_id = $2
        ) AS s ON u.id = s.user_id
        WHERE LOWER(u.username) LIKE '%' || LOWER($1) || '%' AND s.user_id IS NULL
        "#,
        search_username,
        book_id
    )
    .fetch_all(pool)
    .await
}

pub async fn add_user_to_book(
    user_id: i32,
    book_id: i32,
    pool: &PgPool,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (user_id, book_id, role)
        VALUES ($1, $2, to_jsonb('participant'::TEXT))
        ON CONFLICT (user_id, book_id)
        DO NOTHING
        RETURNING user_id
        "#,
        user_id,
        book_id
    )
    .fetch_optional(pool)
    .await
    .map(|row| row.map(|r| r.user_id))
}

pub async fn remove_user_from_book(
    user_id: i32,
    book_id: i32,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM subscriptions
        WHERE user_id = $1 AND book_id = $2
        "#,
        user_id,
        book_id
    )
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn delete_book_cascade(book_id: i32, pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    sqlx::query!(r#"DELETE FROM picks WHERE book_id = $1"#, book_id)
        .execute(&mut *transaction)
        .await?;

    sqlx::query!(r#"DELETE FROM events WHERE book_id = $1"#, book_id)
        .execute(&mut *transaction)
        .await?;

    sqlx::query!(r#"DELETE FROM chapters WHERE book_id = $1"#, book_id)
        .execute(&mut *transaction)
        .await?;

    sqlx::query!(r#"DELETE FROM subscriptions WHERE book_id = $1"#, book_id)
        .execute(&mut *transaction)
        .await?;

    sqlx::query!(r#"DELETE FROM books WHERE id = $1"#, book_id)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;
    Ok(())
}
