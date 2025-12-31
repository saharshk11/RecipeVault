use backend::auth;
use secrecy::ExposeSecret;

mod test_utils;

#[tokio::test]
async fn hash_and_verify_password() {
    let password = "test_password_123";
    let hash = auth::hash_password(password).expect("hash failed");
    let ok = auth::verify_password(&hash, password).expect("verify failed");
    let bad = auth::verify_password(&hash, "wrong").expect("verify failed");
    assert!(ok);
    assert!(!bad);
}

#[tokio::test]
async fn create_and_verify_user() {
    let pool = test_utils::test_db().await;
    let user = auth::create_user(
        &pool,
        "admin_user",
        "secret-pass",
        auth::ROLE_ADMIN,
        true,
    )
    .await
    .expect("create user failed");

    let fetched = auth::find_user_by_id(&pool, &user.id)
        .await
        .expect("find user failed")
        .expect("missing user");

    assert_eq!(fetched.username, "admin_user");
    assert!(fetched.must_change_password);

    let verified = auth::verify_credentials(&pool, "admin_user", "secret-pass")
        .await
        .expect("verify credentials failed")
        .expect("no user");
    assert_eq!(verified.id, user.id);
}

#[tokio::test]
async fn update_credentials_clears_reset_flag() {
    let pool = test_utils::test_db().await;
    let user = auth::create_user(
        &pool,
        "admin_user",
        "secret-pass",
        auth::ROLE_ADMIN,
        true,
    )
    .await
    .expect("create user failed");

    let updated = auth::update_user_credentials(
        &pool,
        &user.id,
        "new_admin",
        "new-pass",
        auth::ROLE_ADMIN,
        false,
    )
    .await
    .expect("update credentials failed");

    assert_eq!(updated.username, "new_admin");
    assert!(!updated.must_change_password);

    let verified = auth::verify_credentials(&pool, "new_admin", "new-pass")
        .await
        .expect("verify credentials failed")
        .expect("no user");
    assert_eq!(verified.id, user.id);
}

#[tokio::test]
async fn sessions_work_and_expire() {
    let pool = test_utils::test_db().await;
    let user = auth::create_user(
        &pool,
        "admin_user",
        "secret-pass",
        auth::ROLE_ADMIN,
        false,
    )
    .await
    .expect("create user failed");

    let session = auth::create_session(&pool, &user.id, 60)
        .await
        .expect("create session failed");
    let by_session = auth::find_user_by_session_id(&pool, &session.id)
        .await
        .expect("find user by session failed");
    assert!(by_session.is_some());

    let expired = auth::create_session(&pool, &user.id, -60)
        .await
        .expect("create expired session failed");
    let by_expired = auth::find_user_by_session_id(&pool, &expired.id)
        .await
        .expect("find user by expired session failed");
    assert!(by_expired.is_none());
}

#[tokio::test]
async fn admin_bootstrap_generates_credentials_when_missing() {
    let pool = test_utils::test_db().await;
    let bootstrap = auth::ensure_admin_user(&pool, None, None)
        .await
        .expect("bootstrap failed");

    assert_eq!(bootstrap.user.role, auth::ROLE_ADMIN);
    assert!(bootstrap.user.must_change_password);
    assert!(bootstrap.generated_credentials.is_some());

    let creds = bootstrap.generated_credentials.unwrap();
    let verified = auth::verify_credentials(
        &pool,
        &creds.username,
        creds.password.expose_secret(),
    )
    .await
    .expect("verify credentials failed");
    assert!(verified.is_some());
}

#[tokio::test]
async fn admin_bootstrap_respects_provided_credentials() {
    let pool = test_utils::test_db().await;
    let bootstrap = auth::ensure_admin_user(
        &pool,
        Some("fixed_admin"),
        Some("fixed_pass"),
    )
    .await
    .expect("bootstrap failed");

    assert_eq!(bootstrap.user.username, "fixed_admin");
    assert!(!bootstrap.user.must_change_password);
    assert!(bootstrap.generated_credentials.is_none());

    let verified = auth::verify_credentials(&pool, "fixed_admin", "fixed_pass")
        .await
        .expect("verify credentials failed");
    assert!(verified.is_some());
}
