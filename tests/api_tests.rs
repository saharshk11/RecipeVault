use backend::app::build_app;
use backend::auth;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header::{CONTENT_TYPE, COOKIE, SET_COOKIE}};
use axum::Router;
use serde_json::Value;
use tower::ServiceExt;

const TEST_HTML: &str = r#"
    <!doctype html>
    <html lang="en">
      <head>
        <meta charset="utf-8">
        <title>Test Pancakes</title>
        <script type="application/ld+json">
        {
          "@context": "https://schema.org",
          "@type": "Recipe",
          "name": "Test Pancakes",
          "description": "Fluffy test pancakes for unit testing.",
          "recipeIngredient": [
            "1 cup flour",
            "2 tbsp sugar",
            "1 cup milk"
          ],
          "recipeInstructions": [
            {
              "@type": "HowToStep",
              "text": "Mix all ingredients in a bowl."
            },
            {
              "@type": "HowToStep",
              "text": "Cook on a hot griddle until golden."
            }
          ],
          "prepTime": "PT10M",
          "cookTime": "PT20M",
          "totalTime": "PT30M",
          "recipeYield": "4 servings"
        }
        </script>
      </head>
      <body>
        <h1>Test Pancakes</h1>
      </body>
    </html>
"#;

mod test_utils;

async fn start_html_server() -> (String, tokio::task::JoinHandle<()>) {
    let app = Router::new().route("/recipe", axum::routing::get(|| async { TEST_HTML }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind test server");
    let addr = listener.local_addr().expect("missing addr");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve failed");
    });
    (format!("http://{}/recipe", addr), handle)
}

async fn read_body_json(response: axum::response::Response) -> Value {
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.expect("read body");
    serde_json::from_slice(&bytes).expect("invalid json")
}

async fn read_body_string(response: axum::response::Response) -> String {
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.expect("read body");
    String::from_utf8(bytes.to_vec()).expect("invalid utf8")
}

async fn login(app: Router, username: &str, password: &str) -> String {
    let payload = serde_json::json!({
        "username": username,
        "password": password
    });
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.expect("login request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let set_cookie = response
        .headers()
        .get(SET_COOKIE)
        .expect("missing set-cookie")
        .to_str()
        .expect("bad set-cookie");
    set_cookie.split(';').next().unwrap().to_string()
}

#[tokio::test]
async fn health_endpoint() {
    let state = test_utils::test_state().await;
    let app = build_app(state);
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .expect("health request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = read_body_string(response).await;
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn auth_endpoints() {
    let state = test_utils::test_state().await;
    let app = build_app(state.clone());

    auth::create_user(&state.db, "admin", "secret", auth::ROLE_ADMIN, true)
        .await
        .expect("create user");

    let cookie = login(app.clone(), "admin", "secret").await;

    let request = Request::builder()
        .uri("/auth/me")
        .header(COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("me request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let body = read_body_json(response).await;
    assert_eq!(body["must_change_password"], true);

    let payload = serde_json::json!({
        "current_password": "secret",
        "new_username": "admin2",
        "new_password": "secret2"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/auth/change-credentials")
        .header(CONTENT_TYPE, "application/json")
        .header(COOKIE, cookie.clone())
        .body(Body::from(payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("change request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let body = read_body_json(response).await;
    assert_eq!(body["username"], "admin2");
    assert_eq!(body["must_change_password"], false);

    let request = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header(COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.expect("logout request failed");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn recipes_blocked_until_password_reset() {
    let state = test_utils::test_state().await;
    let app = build_app(state.clone());

    auth::create_user(&state.db, "admin", "secret", auth::ROLE_ADMIN, true)
        .await
        .expect("create user");

    let cookie = login(app.clone(), "admin", "secret").await;

    let request = Request::builder()
        .uri("/recipes")
        .header(COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.expect("recipes request failed");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = read_body_json(response).await;
    assert_eq!(body["error"]["code"], "PASSWORD_RESET_REQUIRED");
}

#[tokio::test]
async fn recipe_endpoints() {
    let state = test_utils::test_state().await;
    let app = build_app(state.clone());

    auth::create_user(&state.db, "admin", "secret", auth::ROLE_ADMIN, false)
        .await
        .expect("create user");

    let cookie = login(app.clone(), "admin", "secret").await;

    let (recipe_url, server_handle) = start_html_server().await;

    let payload = serde_json::json!({ "url": recipe_url });
    let request = Request::builder()
        .method("POST")
        .uri("/parse")
        .header(CONTENT_TYPE, "application/json")
        .header(COOKIE, cookie.clone())
        .body(Body::from(payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("parse request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let parsed = read_body_json(response).await;
    assert_eq!(parsed["title"], "Test Pancakes");

    let payload = serde_json::json!({ "url": recipe_url });
    let request = Request::builder()
        .method("POST")
        .uri("/recipes/import")
        .header(CONTENT_TYPE, "application/json")
        .header(COOKIE, cookie.clone())
        .body(Body::from(payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("import request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let imported = read_body_json(response).await;
    let recipe_id = imported["id"].as_str().expect("missing id").to_string();

    let request = Request::builder()
        .uri("/recipes")
        .header(COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("list request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let list = read_body_json(response).await;
    assert!(list.as_array().unwrap().iter().any(|item| item["id"] == recipe_id));

    let request = Request::builder()
        .uri(format!("/recipes/{recipe_id}"))
        .header(COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("get request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = read_body_json(response).await;
    assert_eq!(fetched["id"], recipe_id);

    let payload = serde_json::json!({ "title": "Renamed", "tags": ["breakfast"] });
    let request = Request::builder()
        .method("PATCH")
        .uri(format!("/recipes/{recipe_id}"))
        .header(CONTENT_TYPE, "application/json")
        .header(COOKIE, cookie.clone())
        .body(Body::from(payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("patch request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let patched = read_body_json(response).await;
    assert_eq!(patched["recipe"]["title"], "Renamed");
    assert_eq!(patched["recipe"]["tags"][0], "breakfast");

    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/recipes/{recipe_id}"))
        .header(COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("delete request failed");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let request = Request::builder()
        .uri(format!("/recipes/{recipe_id}"))
        .header(COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.expect("get request failed");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    server_handle.abort();
}
