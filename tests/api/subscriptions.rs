use hyper::StatusCode;
use sqlx;

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = "name=Le%20Guin&email=ursula_le_guin%40gmail.com";
    let response = test_app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "Le Guin");
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=Le%20Guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_422_when_fields_are_present_but_invalid() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = test_app.post_subscriptions(body.into()).await;

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not return a 422 Unprocessable Entity when the payload was {}.",
            description
        );
    }
}
