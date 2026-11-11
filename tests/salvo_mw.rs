#![cfg(feature = "salvo")]
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};
use xsia_auditing::{AuditContext, salvo::AuditMiddleware};

#[handler]
async fn set_user(depot: &mut Depot) {
    depot.insert("uid", "42".to_string());
}

#[handler]
async fn show() -> String {
    let c = AuditContext::current().expect("context set");
    format!("{:?}|{:?}|{}", c.user_id, c.user_agent, c.url.unwrap())
}

#[tokio::test]
async fn middleware_sets_context() {
    let router = Router::new()
        .hoop(set_user)
        .hoop(AuditMiddleware::new(|d| d.get::<String>("uid").ok().map(|u| ("user".into(), u.clone()))))
        .push(Router::with_path("x").get(show));
    let svc = Service::new(router);
    let body = TestClient::get("http://127.0.0.1/x?a=1")
        .add_header("user-agent", "ua-test", true)
        .send(&svc).await.take_string().await.unwrap();
    assert!(body.contains("Some(\"42\")") && body.contains("ua-test") && body.contains("/x?a=1"), "{body}");
}
