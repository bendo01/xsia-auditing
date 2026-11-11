# xsia-auditing

Model auditing for [SeaORM](https://www.sea-ql.org/SeaORM/) 2.x, inspired by
[owen-it/laravel-auditing](https://github.com/owen-it/laravel-auditing). Works with
Postgres, MySQL and SQLite (enable the matching `postgres` / `mysql` / `sqlite` feature),
with optional [Salvo](https://salvo.rs) (`salvo`) and [Loco](https://loco.rs) (`loco`) integration.

## Setup
1. Add `xsia_auditing::migration::Migration` to your migrator.
2. Implement `Auditable` on your entity `Model` (it must derive `Serialize`):

```rust
impl Auditable for Model {
    fn auditable_type() -> &'static str { "post" }
    fn auditable_id(&self) -> String { self.id.to_string() }
    fn exclude() -> &'static [&'static str] { &["views"] }
    fn transform(d: &mut AuditData) { /* redact secrets */ }
}
```
3. Write through `Auditor` (pass a transaction to make the audit atomic with the change):

```rust
let post = Auditor::insert(&db, active).await?;
let post2 = Auditor::update(&db, &post, active_changes).await?;
Auditor::delete::<_, post::ActiveModel>(&db, post2).await?;
let history = Auditor::for_model::<_, post::Model>(&db, post.id).await?;
Auditor::prune(&db, cutoff).await?;
```

Updates only store changed, audited attributes; an update that changes nothing audited stores no row.

## Request context
Wrap requests so audits get user / url / IP / user-agent:
- Salvo: `router.hoop(salvo::AuditMiddleware::new(|depot| ...))`
- Loco: `Box::new(loco::AuditInitializer::new(|req| ...))` in `App::initializers`
- Elsewhere: `AuditContext { .. }.scope(async { ... }).await`

## Limitations
- Only writes made through `Auditor` are audited; raw `update_many` / `delete_many` and plain
  `ActiveModel::insert/update` bypass it (SeaORM hooks cannot see the pre-update row).
- `update` needs the old model, which you usually already have from the `find`.
- Loco integration is compile-checked but not yet tested inside a real Loco app.
