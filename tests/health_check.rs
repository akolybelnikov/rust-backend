use std::net::TcpListener;
use reqwest::Response;
use zero2prod::startup::run;
use sqlx::{PgConnection, Connection, Row};
use zero2prod::configuration::get_configuration;

/// Spin up an instance of the app and return its address
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn send_post(body: &'static str, url: &str) -> Response {
    // Set up
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    client
        .post(&format!("{}{}", &app_address, &url))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.")
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");
    // Execute request
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = send_post(body, "/subscriptions").await;
    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Execute request
        let response = send_post(invalid_body, "/subscriptions").await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Custom error message when test fails
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}