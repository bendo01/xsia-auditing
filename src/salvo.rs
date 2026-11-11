//! Salvo integration: [`AuditMiddleware`] runs the rest of the request inside an
//! [`AuditContext`] so audits recorded by handlers get user/url/ip/user-agent.
//!
//! ```ignore
//! let router = Router::new()
//!     .hoop(AuditMiddleware::new(|depot| depot.get::<String>("user_id").ok().map(|id| ("user".into(), id.clone()))))
//!     .push(...);
//! ```
use std::sync::Arc;

use salvo::{Depot, FlowCtrl, Handler, Request, Response, async_trait};

use crate::AuditContext;

type UserResolver = dyn Fn(&Depot) -> Option<(String, String)> + Send + Sync;

#[derive(Clone)]
pub struct AuditMiddleware {
    resolver: Arc<UserResolver>,
}

impl AuditMiddleware {
    /// `resolver` returns `(user_type, user_id)` for the authenticated user.
    pub fn new(resolver: impl Fn(&Depot) -> Option<(String, String)> + Send + Sync + 'static) -> Self {
        Self { resolver: Arc::new(resolver) }
    }

    /// No user resolution (anonymous audits with url/ip/user-agent only).
    pub fn anonymous() -> Self {
        Self::new(|_| None)
    }
}

#[async_trait]
impl Handler for AuditMiddleware {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        let user = (self.resolver)(depot);
        let ctx = AuditContext {
            user_type: user.as_ref().map(|u| u.0.clone()),
            user_id: user.map(|u| u.1),
            url: Some(req.uri().to_string()),
            ip_address: req.remote_addr().to_string().into(),
            user_agent: req.header::<String>("user-agent"),
            tags: Vec::new(),
        };
        ctx.scope(ctrl.call_next(req, depot, res)).await;
    }
}
