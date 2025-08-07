use crate::model::book::BookRole;
use crate::model::chapter::get_chapter_users;
use crate::model::event::{get_chapter_picks, get_events, get_picks};
use crate::model::team::get_chapter_teams;

use crate::AppNotification;
use crate::{
    auth::{AuthSession, BackendPgDB},
    model::{book::BookSubscription, chapter::Chapter},
    AppError,
};

use axum::{Extension, Json};
use axum_ctx::{RespErr, RespErrCtx, RespErrExt, StatusCode};
use itertools::Itertools;

pub async fn open_book(
    auth_session: AuthSession,
    book_subscription: &BookSubscription,
    chapter: &Chapter,
) -> Result<maud::Markup, RespErr> {
    let user = auth_session.user.ok_or(AppError::BackendUser)?;
    let BackendPgDB(pool) = auth_session.backend;

    let user_picks = get_picks(user.id, chapter.chapter_id, &pool);
    let relevent_teams = get_chapter_teams(chapter.chapter_id, &pool);

    let user_picks = user_picks.await.map_err(AppError::from)?;
    let relevent_teams = relevent_teams.await.map_err(AppError::from)?;

    Ok(crate::view::chapter::open::m(
        &user.username,
        &book_subscription.name,
        chapter,
        user_picks,
        book_subscription.role == BookRole::Admin,
        relevent_teams,
    ))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PickSubmission {
    events: Vec<SubmissionEvent>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case",
    tag = "type"
)]
pub enum SubmissionEvent {
    SpreadGroup {
        event_id: String,
        spreads: Vec<SpreadGroupSpread>,
    },
    UserInput {
        user_input: String,
        event_id: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SpreadGroupSpread {
    num_points: String,
    selection: String,
}

pub async fn submit(
    auth_session: AuthSession,
    Extension(chapter): Extension<Chapter>,
    picks: Result<Json<PickSubmission>, axum::extract::rejection::JsonRejection>,
) -> Result<AppNotification, AppNotification> {
    let Ok(Json(picks)) = picks else {
        tracing::debug!("Could not deserialize picks: {picks:?}");
        return Err(AppNotification(
            StatusCode::BAD_REQUEST,
            "Can't Process Picks. Are they all the way filled out?".into(),
        ));
    };

    let user_id = auth_session.user.ok_or(AppError::BackendUser)?.id;
    let pool = auth_session.backend.0;

    let (event_ids, choices, wagers) = validate_picks(picks.events, &pool).await?;

    sqlx::query!(
        r#"
        INSERT INTO picks (book_id, chapter_id, user_id, event_id, choice, wager)
        SELECT $1 AS book_id, $2 AS chapter_id, $3 AS user_id, event_id, choice, wager
        FROM UNNEST($4::INT[], $5::JSONB[], $6::JSONB[]) AS a(event_id, choice, wager)
        ON CONFLICT (book_id, chapter_id, event_id, user_id)
        DO UPDATE SET
            choice = EXCLUDED.choice,
            wager = EXCLUDED.wager
        "#,
        chapter.book_id,
        chapter.chapter_id,
        user_id,
        &event_ids,
        &choices,
        &wagers
    )
    .execute(&pool)
    .await
    .map_err(AppError::from)?;

    Ok(AppNotification(StatusCode::OK, "Picks Saved".into()))
}

async fn validate_picks(
    events: Vec<SubmissionEvent>,
    pool: &sqlx::PgPool,
) -> Result<(Vec<i32>, Vec<serde_json::Value>, Vec<serde_json::Value>), RespErr> {
    let (events, choices, wagers) = events
        .into_iter()
        .map(|event| match event {
            SubmissionEvent::SpreadGroup { event_id, spreads } => {
                let (min_points, max_points) = (1, spreads.len() as i32);
                let mut point_choices = vec![0; spreads.len()];
                let (choices, wagers) =
                        spreads
                            .into_iter()
                            .map(|spread| {
                                let amount = spread
                                            .num_points
                                            .parse::<i32>()
                                            .ctx(StatusCode::BAD_REQUEST)
                                            .user_msg("Could not parse Spread Group Points")?;

                                if amount < min_points || amount > max_points {
                                    return Err(RespErr::new(StatusCode::BAD_REQUEST).user_msg(format!("Points must be in range {min_points}-{max_points}")))
                                }

                                point_choices[amount as usize - 1] += 1;
                                Ok((
                                    serde_json::Value::String(spread.selection),
                                    serde_json::Value::Number(amount.into()),
                                ))
                            })
                            .try_fold(
                                (Vec::new(), Vec::new()),
                                |(mut choices, mut wagers),
                                 curr_item: Result<
                                    (serde_json::Value, serde_json::Value),
                                    RespErr,
                                >| {
                                    let (curr_choice, curr_wager) = curr_item?;
                                    choices.push(curr_choice);
                                    wagers.push(curr_wager);
                                    Ok::<_, RespErr>((choices, wagers))
                                },
                            )?;
                    let double_used = point_choices.iter().enumerate().filter_map(|(i, count)| (*count > 1).then_some(i+1)).join(",");
                    if !double_used.is_empty() {
                        let unused = point_choices.iter().enumerate().filter_map(|(i, count)| (*count < 1).then_some(i+1)).join(",");
                        return Err(RespErr::new(StatusCode::BAD_REQUEST).user_msg(format!("Points Used<br/>Multiple Times: {double_used}<br/>Point(s) available: {unused}")))
                    }
                Ok((
                    event_id,
                    serde_json::Value::Array(choices),
                    serde_json::Value::Array(wagers),
                ))
            }
            SubmissionEvent::UserInput {
                user_input,
                event_id,
            } => Ok((
                event_id,
                serde_json::Value::String(user_input),
                serde_json::Value::Number(1.into()),
            )),
        })
        .try_fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut events, mut choices, mut wagers),
             curr_item: Result<
                (String, serde_json::Value, serde_json::Value),
                RespErr,
            >| {
                let (curr_event, curr_choices, curr_wager) = curr_item?;
                events.push(curr_event);
                choices.push(curr_choices);
                wagers.push(curr_wager);
                Ok::<_, RespErr>((events, choices, wagers))
            },
        )?;

    let event_ids = events
        .iter()
        .map(|event| event.parse::<i32>())
        .collect::<Result<Vec<_>, _>>()
        .ctx(StatusCode::BAD_REQUEST)
        .user_msg("Could not parse event id")
        .log_msg("Could not parse event id")?;

    let unknown_events = sqlx::query!(
        r#"
        SELECT a
        FROM UNNEST($1::INT[]) AS a
        LEFT JOIN events on a = events.id
        WHERE events.id IS NULL
    "#,
        &event_ids
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    if !unknown_events.is_empty() {
        return Err(RespErr::new(StatusCode::BAD_REQUEST).user_msg("Event not found"));
    }

    Ok((event_ids, choices, wagers))
}

pub async fn closed_book(
    auth_session: AuthSession,
    book_subscription: &BookSubscription,
    chapter: &Chapter,
) -> Result<maud::Markup, RespErr> {
    let curr_user = auth_session
        .user
        .ok_or(RespErr::new(StatusCode::INTERNAL_SERVER_ERROR))?;
    let pool = auth_session.backend.0;

    let events = get_events(chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    let relevent_teams = get_chapter_teams(chapter.chapter_id, &pool)
        .await
        .map_err(AppError::from)?;

    let users = get_chapter_users(book_subscription.id, chapter.chapter_id, &pool).await?;

    let user_picks = get_chapter_picks(chapter.chapter_id, &pool).await?;

    Ok(crate::view::chapter::closed::m(
        curr_user,
        chapter,
        book_subscription,
        &users,
        &user_picks,
        &events,
        &relevent_teams,
    ))
}
