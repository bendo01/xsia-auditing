use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuditEvent {
    Created,
    Updated,
    Deleted,
    Restored,
}

impl AuditEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Deleted => "deleted",
            Self::Restored => "restored",
        }
    }
}

/// Old / new attribute values about to be stored. Mutate in [`Auditable::transform`]
/// to redact or mask values.
#[derive(Clone, Debug, Default)]
pub struct AuditData {
    pub old_values: Map<String, Value>,
    pub new_values: Map<String, Value>,
}

/// Implement on a SeaORM entity `Model` (which must derive `Serialize`).
pub trait Auditable: Serialize {
    /// Stored in `auditable_type` (Laravel's morph class).
    fn auditable_type() -> &'static str;
    /// Stored in `auditable_id`.
    fn auditable_id(&self) -> String;
    /// If non-empty, only these attributes are audited.
    fn include() -> &'static [&'static str] {
        &[]
    }
    /// Attributes never audited (applied after `include`).
    fn exclude() -> &'static [&'static str] {
        &[]
    }
    /// Events to record.
    fn events() -> &'static [AuditEvent] {
        &[AuditEvent::Created, AuditEvent::Updated, AuditEvent::Deleted]
    }
    /// Extra tags for this audit (joined with `,`).
    fn tags(&self) -> Vec<String> {
        Vec::new()
    }
    /// Last chance to redact/modify values.
    fn transform(_data: &mut AuditData) {}
}
