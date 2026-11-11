use sea_orm::{Database, DatabaseConnection, Set, entity::prelude::*, IntoActiveModel};
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use serde::{Deserialize, Serialize};
use xsia_auditing::{AuditContext, AuditData, Auditable, Auditor};

mod post {
    use super::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "posts")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub title: String,
        pub secret: String,
        pub views: i32,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}

    impl Auditable for Model {
        fn auditable_type() -> &'static str { "post" }
        fn auditable_id(&self) -> String { self.id.to_string() }
        fn exclude() -> &'static [&'static str] { &["views"] }
        fn transform(d: &mut AuditData) {
            for m in [&mut d.old_values, &mut d.new_values] {
                if m.contains_key("secret") { m.insert("secret".into(), "***".into()); }
            }
        }
    }
}

struct Migrator;
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(xsia_auditing::migration::Migration)]
    }
}

async fn setup() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db.execute_unprepared("CREATE TABLE posts (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, secret TEXT NOT NULL, views INTEGER NOT NULL)")
        .await.unwrap();
    db
}

use sea_orm::ConnectionTrait;

#[tokio::test]
async fn insert_update_delete_are_audited() {
    let db = setup().await;
    let ctx = AuditContext { user_type: Some("user".into()), user_id: Some("7".into()), url: Some("/posts".into()), ..Default::default() };
    ctx.scope(async {
        let p = Auditor::insert(&db, post::ActiveModel {
            title: Set("a".into()), secret: Set("pw".into()), views: Set(0), ..Default::default()
        }).await.unwrap();

        // exclude-only change => no audit
        let mut am = p.clone().into_active_model();
        am.views = Set(5);
        let p2 = Auditor::update(&db, &p, am).await.unwrap();

        let mut am = p2.clone().into_active_model();
        am.title = Set("b".into());
        let p3 = Auditor::update(&db, &p2, am).await.unwrap();

        Auditor::delete::<_, post::ActiveModel>(&db, p3.clone()).await.unwrap();

        let rows = Auditor::for_model::<_, post::Model>(&db, p.id).await.unwrap();
        let events: Vec<_> = rows.iter().rev().map(|r| r.event.as_str()).collect();
        assert_eq!(events, ["created", "updated", "deleted"]);
        assert_eq!(rows[2].user_id.as_deref(), Some("7"));
        assert_eq!(rows[2].url.as_deref(), Some("/posts"));
        let upd = &rows[1];
        assert_eq!(upd.old_values.as_ref().unwrap(), &serde_json::json!({"title": "a"}));
        assert_eq!(upd.new_values.as_ref().unwrap(), &serde_json::json!({"title": "b"}));
        let created = &rows[2];
        assert_eq!(created.new_values.as_ref().unwrap()["secret"], "***");
        assert!(created.new_values.as_ref().unwrap().get("views").is_none());
    }).await;
}

#[tokio::test]
async fn prune_removes_old_rows() {
    let db = setup().await;
    Auditor::insert(&db, post::ActiveModel { title: Set("a".into()), secret: Set("x".into()), views: Set(0), ..Default::default() }).await.unwrap();
    let n = Auditor::prune(&db, chrono::Utc::now() + chrono::Duration::seconds(1)).await.unwrap();
    assert_eq!(n, 1);
}
