use axum_core::response::IntoResponse;
use axum_ctx::*;
use http::StatusCode;

#[test]
fn ok() {
    let res: Result<u64, &'static str> = Ok(42);
    let res = res.ctx(StatusCode::BAD_REQUEST);

    assert!(matches!(res, Ok(42)));
}

#[test]
fn err() {
    let res: Result<u64, &'static str> = Err("Ups!");
    let res = res
        .ctx(StatusCode::BAD_REQUEST)
        .map_err(|e| e.into_response());

    assert!(res.is_err_and(|e| e.status() == StatusCode::BAD_REQUEST));
}

#[test]
fn err_as_log_msg() {
    let err_content = "Ups!";
    let res: Result<u64, &'static str> = Err(err_content);
    let res = res.ctx(StatusCode::BAD_REQUEST);

    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        format!("{}\n  0: {err_content}", StatusCode::BAD_REQUEST)
    );
}

#[test]
fn err_as_log_msg_with_additional_log_msg() {
    let err_content = "Ups!";
    let res: Result<u64, &'static str> = Err(err_content);
    let log_msg = "Nooo!";
    let res = res.ctx(StatusCode::BAD_REQUEST).log_msg(log_msg);

    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        format!(
            "{}\n  0: {log_msg}\n  1: {err_content}",
            StatusCode::BAD_REQUEST,
        )
    );
}

#[test]
fn some_user_msg() {
    let opt = Some(42);
    let res = opt.ctx(StatusCode::INTERNAL_SERVER_ERROR);

    assert!(matches!(res, Ok(42)));
}

#[test]
fn none() {
    let opt: Option<u64> = None;
    let res = opt
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .map_err(|e| e.into_response());

    assert!(res.is_err_and(|e| e.status() == StatusCode::INTERNAL_SERVER_ERROR));
}

#[test]
fn default_user_msg() {
    let opt: Option<u64> = None;
    let res = opt.ctx(StatusCode::INTERNAL_SERVER_ERROR);

    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR.to_string(),
    );
}

#[test]
fn custom_user_msg() {
    let opt: Option<u64> = None;
    let user_msg = "Missing number!";
    let res = opt
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .user_msg(user_msg);
    let err = res.unwrap_err();

    assert_eq!(err.to_string(), user_msg);
}

#[test]
fn default_user_msg_with_one_log_msg() {
    let opt: Option<u64> = None;
    let log_msg = "Bug!";
    let res = opt.ctx(StatusCode::INTERNAL_SERVER_ERROR).log_msg(log_msg);

    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        format!("{}\n  0: {log_msg}", StatusCode::INTERNAL_SERVER_ERROR)
    );
}

#[test]
fn default_user_msg_with_two_log_msgs() {
    let opt: Option<u64> = None;
    let first_log_msg = "Bug!";
    let second_log_msg = "Bugs everywhere!";
    let res = opt
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .log_msg(first_log_msg)
        .log_msg(second_log_msg);

    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        format!(
            "{}\n  0: {second_log_msg}\n  1: {first_log_msg}",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    );
}

#[test]
fn custom_user_msg_with_two_log_msgs() {
    let opt: Option<u64> = None;
    let user_msg = "Sorry!";
    let first_log_msg = "Bug!";
    let second_log_msg = "Bugs everywhere!";
    let res = opt
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .user_msg(user_msg)
        .log_msg(first_log_msg)
        .log_msg(second_log_msg);

    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        format!("{user_msg}\n  0: {second_log_msg}\n  1: {first_log_msg}")
    );
}

#[test]
fn closures() {
    let opt: Option<u64> = None;
    let n = 42;
    let user_msg = || format!("Sorry for the {n}th bug!");
    let first_log_msg = || format!("{n} times!");
    let second_log_msg = ":(";
    let res = opt
        .ctx(StatusCode::INTERNAL_SERVER_ERROR)
        .user_msg(user_msg)
        .log_msg(first_log_msg)
        .log_msg(second_log_msg);

    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        format!(
            "{}\n  0: {second_log_msg}\n  1: {}",
            user_msg(),
            first_log_msg(),
        )
    );
}
