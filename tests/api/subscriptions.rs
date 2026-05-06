use crate::helpers::{spawn_app, TestApp};

#[tokio::test]
async fn subscrive_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app().await;
    let connection_pool = app_address.clone().db_pool;
    let body = "name=le%20guin&email=ursula_le_guin%40example.com";
   let response=app_address.post_subscriptions(body.to_string()).await;
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&connection_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "ursula_le_guin@example.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscrive_returns_a_400_for_invalid_form_data() {
    let app_address = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing email"),
        ("", "missing name and email"),
        ("email=ursula_le_guin%40example.com", "missing name"),
    ];
    for (invalid_body, _error_message) in test_cases {
        let response = app_address.post_subscriptions(invalid_body.to_string()).await;
        assert_eq!(400, response.status().as_u16());
    }
}
