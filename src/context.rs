//! Request-scoped audit context carried in a task-local.
use std::future::Future;

#[derive(Clone, Debug, Default)]
pub struct AuditContext {
    pub user_type: Option<String>,
    pub user_id: Option<String>,
    pub url: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub tags: Vec<String>,
}

tokio::task_local! {
    static CURRENT: AuditContext;
}

impl AuditContext {
    /// Run `fut` with this context active. Audits recorded inside use it.
    pub async fn scope<F: Future>(self, fut: F) -> F::Output {
        CURRENT.scope(self, fut).await
    }

    /// The active context, if inside [`AuditContext::scope`].
    pub fn current() -> Option<AuditContext> {
        CURRENT.try_with(|c| c.clone()).ok()
    }
}
