pub async fn user_exists(username: &str, pool: &sqlx::PgPool) -> Result<bool, sqlx::Error> {
    sqlx::query!(
        "
		SELECT id
		FROM users
		WHERE username = $1
		",
        username
    )
    .fetch_optional(pool)
    .await
    .map(|row| row.is_some())
}
