use chrono::Utc;
use uuid::Uuid;

use super::*;

#[test]
fn should_create_create_stmt() {
    let expected_stmt = r#"create table if not exists users( id bigserial primary key, created_at timestamptz not null default now(), updated_at timestamptz, name text unique not null )"#;
    let coll = Collection {
        name: "users".into(),
        column_defs: vec![ColumnDef {
            id: Uuid::new_v4(),
            name: "name".into(),
            column_type: ColumnType::Text,
            required: true,
            unique: true,
        }],
    };
    let ct_stmt = coll.create_table_statement();
    assert_eq!(expected_stmt, ct_stmt, "create statement match failed");
}
#[sqlx::test]
async fn should_add_column_to_collection(pool: sqlx::PgPool) {
    #[derive(sqlx::FromRow)]
    #[allow(dead_code)]
    struct AddColumnOrg {
        id: i64,
        name: String,
        website: String,
        created_at: chrono::DateTime<Utc>,
        updated_at: Option<chrono::DateTime<Utc>>,
    }

    let mut conn = pool.acquire().await.expect("unable to get a connection");
    let name_id = Uuid::new_v4();
    let coll = Collection {
        name: "organizations".into(),
        column_defs: vec![ColumnDef {
            id: name_id,
            name: "name".into(),
            column_type: ColumnType::Text,
            required: true,
            unique: true,
        }],
    };
    let new_def = Collection {
        name: "organizations".into(),
        column_defs: vec![
            ColumnDef {
                id: name_id,
                name: "name".into(),
                column_type: ColumnType::Text,
                required: true,
                unique: true,
            },
            ColumnDef {
                id: Uuid::new_v4(),
                name: "website".into(),
                column_type: ColumnType::Text,
                required: true,
                unique: true,
            },
        ],
    };
    coll.create_collection(&mut conn)
        .await
        .expect("unable to create collection");
    coll.update_collection(&mut conn, &new_def)
        .await
        .expect("Could not update collection");
    let res = conn
        .execute("insert into organizations(name, website) values('tarkalabs', 'tarkalabs.com')")
        .await
        .expect("unable to insert org");
    assert_eq!(
        1,
        res.rows_affected(),
        "expected one row to be affected. affected {} rows",
        res.rows_affected()
    );
    let orgs = sqlx::query_as::<_, AddColumnOrg>("select * from organizations")
        .fetch_all(&mut conn)
        .await
        .expect("unable to query orgs");

    assert_eq!(orgs.len(), 1, "expected 1 row found {} rows", orgs.len());
}
#[sqlx::test]
async fn should_rename_column_in_collection(pool: sqlx::PgPool) {
    #[derive(sqlx::FromRow)]
    #[allow(dead_code)]
    struct RenameColumnOrg {
        id: i64,
        name_new: String,
        created_at: chrono::DateTime<Utc>,
        updated_at: Option<chrono::DateTime<Utc>>,
    }

    let mut conn = pool.acquire().await.expect("unable to get a connection");
    let name_id = Uuid::new_v4();
    let coll = Collection {
        name: "organizations".into(),
        column_defs: vec![ColumnDef {
            id: name_id,
            name: "name".into(),
            column_type: ColumnType::Text,
            required: true,
            unique: true,
        }],
    };
    let new_def = Collection {
        name: "organizations".into(),
        column_defs: vec![
            ColumnDef {
                id: name_id,
                name: "name_new".into(),
                column_type: ColumnType::Text,
                required: true,
                unique: true,
            }
        ],
    };
    coll.create_collection(&mut conn)
        .await
        .expect("unable to create collection");
    coll.update_collection(&mut conn, &new_def)
        .await
        .expect("Could not update collection");
    let res = conn
        .execute("insert into organizations(name_new) values('tarkalabs')")
        .await
        .expect("unable to insert org");
    assert_eq!(
        1,
        res.rows_affected(),
        "expected one row to be affected. affected {} rows",
        res.rows_affected()
    );
    let orgs = sqlx::query_as::<_, RenameColumnOrg>("select * from organizations")
        .fetch_all(&mut conn)
        .await
        .expect("unable to query orgs");

    assert_eq!(orgs.len(), 1, "expected 1 row found {} rows", orgs.len());
	assert_eq!(orgs[0].name_new, "tarkalabs");
}
#[sqlx::test]
async fn should_create_collection(pool: sqlx::PgPool) {
    #[derive(sqlx::FromRow)]
    #[allow(dead_code)]
    struct Organization {
        id: i64,
        name: String,
        created_at: chrono::DateTime<Utc>,
        updated_at: Option<chrono::DateTime<Utc>>,
    }
    let mut conn = pool.acquire().await.expect("unable to get a connection");
    let coll = Collection {
        name: "organizations".into(),
        column_defs: vec![ColumnDef {
            id: Uuid::new_v4(),
            name: "name".into(),
            column_type: ColumnType::Text,
            required: true,
            unique: true,
        }],
    };
    coll.create_collection(&mut conn)
        .await
        .expect("unable to create collection");
    let res = conn
        .execute("insert into organizations(name) values('tarkalabs')")
        .await
        .expect("unable to insert org");
    assert_eq!(
        1,
        res.rows_affected(),
        "expected one row to be affected. affected {} rows",
        res.rows_affected()
    );
    let orgs = sqlx::query_as::<_, Organization>("select * from organizations")
        .fetch_all(&mut conn)
        .await
        .expect("unable to query orgs");

    assert_eq!(orgs.len(), 1, "expected 1 row found {} rows", orgs.len());
}
