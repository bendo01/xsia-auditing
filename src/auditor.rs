use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, Set,
};
use serde_json::{Map, Value};

use crate::{
    AuditContext, AuditData, AuditError, AuditEvent, Auditable,
    entity::{self as audits},
};

/// Entry point for audited writes and audit queries.
pub struct Auditor;

fn to_map<M: Auditable>(m: &M) -> Result<Map<String, Value>, AuditError> {
    let Value::Object(mut map) = serde_json::to_value(m)? else {
        return Ok(Map::new());
    };
    let include = M::include();
    if !include.is_empty() {
        map.retain(|k, _| include.contains(&k.as_str()));
    }
    let exclude = M::exclude();
    map.retain(|k, _| !exclude.contains(&k.as_str()));
    Ok(map)
}

/// Compute the audit payload. Returns `None` when there is nothing to record
/// (event disabled, or an update that changed no audited attribute).
pub fn build_audit_data<M: Auditable>(
    event: AuditEvent,
    old: Option<&M>,
    new: Option<&M>,
) -> Result<Option<AuditData>, AuditError> {
    if !M::events().contains(&event) {
        return Ok(None);
    }
    let old_map = old.map(to_map).transpose()?.unwrap_or_default();
    let new_map = new.map(to_map).transpose()?.unwrap_or_default();
    let mut data = match event {
        AuditEvent::Created | AuditEvent::Restored => AuditData { old_values: Map::new(), new_values: new_map },
        AuditEvent::Deleted => AuditData { old_values: old_map, new_values: Map::new() },
        AuditEvent::Updated => {
            let mut d = AuditData::default();
            for (k, nv) in new_map {
                if old_map.get(&k) != Some(&nv) {
                    if let Some(ov) = old_map.get(&k) {
                        d.old_values.insert(k.clone(), ov.clone());
                    }
                    d.new_values.insert(k, nv);
                }
            }
            if d.new_values.is_empty() {
                return Ok(None);
            }
            d
        }
    };
    M::transform(&mut data);
    Ok(Some(data))
}

impl Auditor {
    /// Store an audit row for `event`. Normally called via insert/update/delete.
    pub async fn record<C, M>(
        db: &C,
        event: AuditEvent,
        old: Option<&M>,
        new: Option<&M>,
    ) -> Result<Option<audits::Model>, AuditError>
    where
        C: ConnectionTrait,
        M: Auditable,
    {
        let Some(subject) = new.or(old) else { return Ok(None) };
        let Some(data) = build_audit_data(event, old, new)? else { return Ok(None) };
        let ctx = AuditContext::current().unwrap_or_default();
        let mut tags = ctx.tags.clone();
        tags.extend(subject.tags());
        let json = |m: Map<String, Value>| (!m.is_empty()).then_some(Value::Object(m));
        let row = audits::ActiveModel {
            user_type: Set(ctx.user_type),
            user_id: Set(ctx.user_id),
            event: Set(event.as_str().to_owned()),
            auditable_type: Set(M::auditable_type().to_owned()),
            auditable_id: Set(subject.auditable_id()),
            old_values: Set(json(data.old_values)),
            new_values: Set(json(data.new_values)),
            url: Set(ctx.url),
            ip_address: Set(ctx.ip_address),
            user_agent: Set(ctx.user_agent),
            tags: Set((!tags.is_empty()).then(|| tags.join(","))),
            created_at: Set(Utc::now()),
            ..Default::default()
        };
        Ok(Some(row.insert(db).await?))
    }

    /// Insert and record a `created` audit. Use a transaction `db` to make both atomic.
    pub async fn insert<C, A>(db: &C, active: A) -> Result<<A::Entity as EntityTrait>::Model, AuditError>
    where
        C: ConnectionTrait,
        A: ActiveModelTrait + ActiveModelBehavior + Send,
        <A::Entity as EntityTrait>::Model: Auditable + IntoActiveModel<A>,
    {
        let model = active.insert(db).await?;
        Self::record(db, AuditEvent::Created, None, Some(&model)).await?;
        Ok(model)
    }

    /// Update and record an `updated` audit diffing `old` against the result.
    pub async fn update<C, A>(
        db: &C,
        old: &<A::Entity as EntityTrait>::Model,
        active: A,
    ) -> Result<<A::Entity as EntityTrait>::Model, AuditError>
    where
        C: ConnectionTrait,
        A: ActiveModelTrait + ActiveModelBehavior + Send,
        <A::Entity as EntityTrait>::Model: Auditable + IntoActiveModel<A>,
    {
        let model = active.update(db).await?;
        Self::record(db, AuditEvent::Updated, Some(old), Some(&model)).await?;
        Ok(model)
    }

    /// Delete and record a `deleted` audit.
    pub async fn delete<C, A>(db: &C, model: <A::Entity as EntityTrait>::Model) -> Result<(), AuditError>
    where
        C: ConnectionTrait,
        A: ActiveModelTrait + ActiveModelBehavior + Send,
        <A::Entity as EntityTrait>::Model: Auditable + IntoActiveModel<A>,
    {
        ModelTrait::delete(model.clone(), db).await?;
        Self::record(db, AuditEvent::Deleted, Some(&model), None).await?;
        Ok(())
    }

    /// Audits for one record, newest first.
    pub async fn for_model<C: ConnectionTrait, M: Auditable>(
        db: &C,
        id: impl ToString,
    ) -> Result<Vec<audits::Model>, AuditError> {
        Ok(audits::Entity::find()
            .filter(audits::Column::AuditableType.eq(M::auditable_type()))
            .filter(audits::Column::AuditableId.eq(id.to_string()))
            .order_by_desc(audits::Column::Id)
            .all(db)
            .await?)
    }

    /// Delete audits older than `before`; returns rows removed.
    pub async fn prune<C: ConnectionTrait>(db: &C, before: DateTime<Utc>) -> Result<u64, AuditError> {
        Ok(audits::Entity::delete_many()
            .filter(audits::Column::CreatedAt.lt(before))
            .exec(db)
            .await?
            .rows_affected)
    }
}
