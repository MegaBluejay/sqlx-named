use anyhow::Result;
use sqlx::PgPool;

#[sqlx::test]
async fn test_query_no_args(db: PgPool) -> Result<()> {
    let account = sqlx_named::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = 1",
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(account.id, Some(1));
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_unnamed(db: PgPool) -> Result<()> {
    let account = sqlx_named::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $1",
        1i32,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(account.id, Some(1));
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_unnamed_ident(db: PgPool) -> Result<()> {
    let id = 1i32;
    let account = sqlx_named::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $1",
        id,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(account.id, Some(1));
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_unnamed_cast(db: PgPool) -> Result<()> {
    let account = sqlx_named::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $1",
        1 as i32,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(account.id, Some(1));
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_named(db: PgPool) -> Result<()> {
    let account = sqlx_named::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $id",
        id = 1i32,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(account.id, Some(1));
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_named_cast(db: PgPool) -> Result<()> {
    let account = sqlx_named::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $id",
        id = 1 as i32,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(account.id, Some(1));
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_named_pun(db: PgPool) -> Result<()> {
    let id = 1i32;
    let account = sqlx_named::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $id",
        id,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(account.id, Some(1));
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_named_pun_cast(db: PgPool) -> Result<()> {
    let id = 1;
    let account = sqlx_named::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $id",
        id as i32,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(account.id, Some(1));
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_file_no_args(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_file!("./tests/test-query-no-args.sql")
        .fetch_one(&db)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_file_unnamed(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_file!("./tests/test-query-unnamed.sql", 1i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_file_unnamed_ident(db: PgPool) -> Result<()> {
    let id = 1i32;
    let account = sqlx_named::query_file!("./tests/test-query-unnamed.sql", id)
        .fetch_one(&db)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_file_unnamed_cast(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_file!("./tests/test-query-unnamed.sql", 1 as i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_file_named(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_file!("./tests/test-query-named.sql", id = 1i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_file_named_cast(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_file!("./tests/test-query-named.sql", id = 1 as i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_file_named_pun(db: PgPool) -> Result<()> {
    let id = 1i32;
    let account = sqlx_named::query_file!("./tests/test-query-named.sql", id)
        .fetch_one(&db)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[sqlx::test]
async fn test_query_file_named_pun_cast(db: PgPool) -> Result<()> {
    let id = 1;
    let account = sqlx_named::query_file!("./tests/test-query-named.sql", id as i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Herp Derpinson"));
    Ok(())
}

#[derive(Debug)]
struct Account {
    id: i32,
    name: Option<String>,
}

#[sqlx::test]
async fn test_query_as_no_args(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_as!(
        Account,
        r#"SELECT id "id!", name from (VALUES (1, null)) accounts(id, name)"#,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(1, account.id);
    assert_eq!(None, account.name);

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_as_unnamed(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_as!(
        Account,
        r#"SELECT id "id!", name from (VALUES (1, $1)) accounts(id, name)"#,
        None as Option<&str>,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(1, account.id);
    assert_eq!(None, account.name);

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_as_unnamed_ident(db: PgPool) -> Result<()> {
    let name: Option<&str> = None;
    let account = sqlx_named::query_as!(
        Account,
        r#"SELECT id "id!", name from (VALUES (1, $1)) accounts(id, name)"#,
        name,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(1, account.id);
    assert_eq!(None, account.name);

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_as_named(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_as!(
        Account,
        r#"SELECT id "id!", name from (VALUES (1, $name)) accounts(id, name)"#,
        name = None as Option<&str>,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(1, account.id);
    assert_eq!(None, account.name);

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_as_named_pun(db: PgPool) -> Result<()> {
    let name: Option<&str> = None;
    let account = sqlx_named::query_as!(
        Account,
        r#"SELECT id "id!", name from (VALUES (1, $name)) accounts(id, name)"#,
        name,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(1, account.id);
    assert_eq!(None, account.name);

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_file_as_no_args(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_file_as!(Account, "./tests/test-query-no-args.sql")
        .fetch_one(&db)
        .await?;

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_file_as_unnamed(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_file_as!(Account, "./tests/test-query-unnamed.sql", 1i32)
        .fetch_one(&db)
        .await?;

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_file_as_unnamed_ident(db: PgPool) -> Result<()> {
    let id = 1i32;
    let account = sqlx_named::query_file_as!(Account, "./tests/test-query-unnamed.sql", id)
        .fetch_one(&db)
        .await?;

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_file_as_named(db: PgPool) -> Result<()> {
    let account = sqlx_named::query_file_as!(Account, "./tests/test-query-named.sql", id = 1i32)
        .fetch_one(&db)
        .await?;

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_file_as_named_pun(db: PgPool) -> Result<()> {
    let id = 1i32;
    let account = sqlx_named::query_file_as!(Account, "./tests/test-query-named.sql", id)
        .fetch_one(&db)
        .await?;

    println!("{account:?}");

    Ok(())
}

#[sqlx::test]
async fn test_query_scalar_no_args(db: PgPool) -> Result<()> {
    let id = sqlx_named::query_scalar!("select 1").fetch_one(&db).await?;

    assert_eq!(id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_scalar_unnamed(db: PgPool) -> Result<()> {
    let id = sqlx_named::query_scalar!("select $1::int", 1i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_scalar_unnamed_cast(db: PgPool) -> Result<()> {
    let id = sqlx_named::query_scalar!("select $1::int", 1 as i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_scalar_unnamed_ident(db: PgPool) -> Result<()> {
    let id = 1i32;
    let id = sqlx_named::query_scalar!("select $1::int", id)
        .fetch_one(&db)
        .await?;

    assert_eq!(id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_scalar_named(db: PgPool) -> Result<()> {
    let id = sqlx_named::query_scalar!("select $id::int", id = 1i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_scalar_named_cast(db: PgPool) -> Result<()> {
    let id = sqlx_named::query_scalar!("select $id::int", id = 1 as i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_scalar_named_pun(db: PgPool) -> Result<()> {
    let id = 1i32;
    let id = sqlx_named::query_scalar!("select $id::int", id)
        .fetch_one(&db)
        .await?;

    assert_eq!(id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_scalar_named_pun_cast(db: PgPool) -> Result<()> {
    let id = 1;
    let id = sqlx_named::query_scalar!("select $id::int", id as i32)
        .fetch_one(&db)
        .await?;

    assert_eq!(id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_many_args_unnamed(db: PgPool) -> Result<()> {
    let rows = sqlx_named::query!(
        "SELECT * from unnest(array[$1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12]::int[]) ids(id)",
        0,1,2,3,4,5,6,7,8,9,10,11
    ).fetch_all(&db).await?;

    for (i, row) in rows.into_iter().enumerate() {
        assert_eq!(Some(i as i32), row.id);
    }

    Ok(())
}

#[sqlx::test]
async fn test_query_many_args_named(db: PgPool) -> Result<()> {
    let two = 1;
    let seven = 6;
    let rows = sqlx_named::query!(
        "SELECT * from unnest(array[$one, $two, $three, $four, $five, $six, $seven, $eight, $nine, $ten, $eleven, $twelve]::int[]) ids(id)",
        one = 0,
        two,
        three = 2,
        four = 3,
        five = 4,
        six = 5,
        seven,
        eight = 7,
        nine = 8,
        ten = 9,
        eleven = 10,
        twelve = 11,
    ).fetch_all(&db).await?;

    for (i, row) in rows.into_iter().enumerate() {
        assert_eq!(Some(i as i32), row.id);
    }

    Ok(())
}

#[derive(PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(transparent)]
struct MyInt4(i32);

#[sqlx::test]
async fn test_query_unnamed_override_exact(db: PgPool) -> Result<()> {
    let my_int = MyInt4(1);
    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $1",
        my_int as MyInt4,
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(record.id, Some(1i32));

    let record = sqlx_named::query!("select $1::int8 as id", 1i32 as i64)
        .fetch_one(&db)
        .await?;
    assert_eq!(record.id, Some(1i64));

    let my_opt_int = Some(MyInt4(1));
    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $1",
        my_opt_int as Option<MyInt4>,
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(record.id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_named_override_exact(db: PgPool) -> Result<()> {
    let my_int = MyInt4(1);
    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $id",
        id = my_int as MyInt4,
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(record.id, Some(1i32));

    let id = MyInt4(1);
    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $id",
        id as MyInt4,
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(record.id, Some(1i32));

    let record = sqlx_named::query!("select $id::int8 as id", id = 1i32 as i64)
        .fetch_one(&db)
        .await?;
    assert_eq!(record.id, Some(1i64));

    let id = 1i32;
    let record = sqlx_named::query!("select $id::int8 as id", id as i64)
        .fetch_one(&db)
        .await?;
    assert_eq!(record.id, Some(1i64));

    let my_opt_int = Some(MyInt4(1));
    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $id",
        id = my_opt_int as Option<MyInt4>,
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(record.id, Some(1i32));

    let id = Some(MyInt4(1));
    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $id",
        id as Option<MyInt4>,
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(record.id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_unnamed_override_wildcard(db: PgPool) -> Result<()> {
    let my_int = MyInt4(1);

    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $1",
        my_int as _,
    )
    .fetch_one(&db)
    .await?;

    assert_eq!(record.id, Some(1i32));

    Ok(())
}

#[sqlx::test]
async fn test_query_named_override_wildcard(db: PgPool) -> Result<()> {
    let my_int = MyInt4(1);
    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $id",
        id = my_int as _,
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(record.id, Some(1i32));

    let id = MyInt4(1);
    let record = sqlx_named::query!(
        "select * from (select 1::int4) records(id) where id = $id",
        id as _,
    )
    .fetch_one(&db)
    .await?;
    assert_eq!(record.id, Some(1i32));

    Ok(())
}
