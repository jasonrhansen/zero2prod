use hyper::StatusCode;

use crate::helpers::{assert_status_code, spawn_app, TestApp};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    // Clear out confirmation emails. So we can check if newsletter emails get sent below.
    app.email_server.lock().unwrap().sends.clear();

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": "<p>Newsletter body as HTML</p>"
    });

    let response = app.post_newsletters(newsletter_request_body).await;
    assert_eq!(response.status(), StatusCode::OK);

    let email_server = app.email_server.lock().unwrap();
    assert_eq!(
        0,
        email_server.sends.len(),
        "The API should not send emails to unconfirmed subscribers"
    );
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    // Clear out confirmation emails. So we can check if newsletter emails get sent below.
    app.email_server.lock().unwrap().sends.clear();

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": "<p>Newsletter body as HTML</p>"
    });

    let response = app.post_newsletters(newsletter_request_body).await;
    assert_eq!(response.status(), StatusCode::OK);

    let email_server = app.email_server.lock().unwrap();
    assert_eq!(
        1,
        email_server.sends.len(),
        "The API should send emails to confirmed subscribers"
    );
}

#[tokio::test]
async fn newsletters_returns_422_for_invalid_data() {
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
            "content": "<p>Newsletter body as HTML</p>",
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        assert_status_code(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            error_message,
        );
    }
}

async fn create_unconfirmed_subscriber(app: &TestApp) {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
}

async fn create_confirmed_subscriber(app: &TestApp) {
    create_unconfirmed_subscriber(app).await;
    let confirmation_link = app.confirmation_link();
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
