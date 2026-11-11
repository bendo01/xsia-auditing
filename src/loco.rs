//! Loco (axum) integration: [`audit_layer`] middleware and [`AuditInitializer`].
//!
//! ```ignore
//! // in your App impl
//! async fn initializers(_: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
//!     Ok(vec![Box::new(AuditInitializer::anonymous())])
//! }
//! // in migration/src/lib.rs
//! Box::new(xsia_auditing::migration::Migration)
//! ```
use std::sync::Arc;

use axum::{Router as AxumRouter, extract::Request, middleware::{self, Next}, response::Response};
use loco_rs::{app::{AppContext, Initializer}, Result};

use crate::AuditContext;

/// Resolves `(user_type, user_id)` from a request (e.g. from JWT claims in extensions).
pub type UserResolver = Arc<dyn Fn(&Request) -> Option<(String, String)> + Send + Sync>;

fn client_ip(req: &Request) -> Option<String> {
    let h = req.headers();
    h.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .or_else(|| h.get("x-real-ip").and_then(|v| v.to_str().ok()))
        .map(|s| s.trim().to_owned())
        .or_else(|| {
            req.extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|c| c.0.ip().to_string())
        })
}

pub async fn audit_middleware(resolver: UserResolver, req: Request, next: Next) -> Response {
    let user = resolver(&req);
    let ctx = AuditContext {
        user_type: user.as_ref().map(|u| u.0.clone()),
        user_id: user.map(|u| u.1),
        url: Some(req.uri().to_string()),
        ip_address: client_ip(&req),
        user_agent: req.headers().get("user-agent").and_then(|v| v.to_str().ok()).map(str::to_owned),
        tags: Vec::new(),
    };
    ctx.scope(next.run(req)).await
}

/// Wrap an axum router so handlers run inside an [`AuditContext`].
pub fn audit_layer(router: AxumRouter, resolver: UserResolver) -> AxumRouter {
    router.layer(middleware::from_fn(move |req, next| audit_middleware(resolver.clone(), req, next)))
}

pub struct AuditInitializer {
    resolver: UserResolver,
}

impl AuditInitializer {
    pub fn new(resolver: impl Fn(&Request) -> Option<(String, String)> + Send + Sync + 'static) -> Self {
        Self { resolver: Arc::new(resolver) }
    }
    pub fn anonymous() -> Self {
        Self::new(|_| None)
    }
}

#[async_trait::async_trait]
impl Initializer for AuditInitializer {
    fn name(&self) -> String {
        "xsia-auditing".to_owned()
    }
    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        Ok(audit_layer(router, self.resolver.clone()))
    }
}
