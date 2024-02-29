use sqlx::{pool::PoolConnection, Sqlite};

use crate::error::Result;

pub async fn create_user(
    user_id: &str,
    name: &str,
    conn: &mut PoolConnection<Sqlite>,
) -> Result<()> {
    sqlx::query!("INSERT INTO user (id, name) SELECT ?, ? WHERE NOT EXISTS (SELECT * FROM user WHERE id = ?)",
        user_id,
        name,
        user_id
    )
    .execute(&mut **conn)
    .await?;

    Ok(())
}

pub async fn create_user_with_conversation(
    user_id: &str,
    name: &str,
    conversation_id: &str,
    conn: &mut PoolConnection<Sqlite>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO user (id, name, conversation_id) VALUES (?, ?, ?)",
        user_id,
        name,
        conversation_id
    )
    .execute(&mut **conn)
    .await?;

    Ok(())
}

pub async fn update_conversation(
    user_id: &str,
    conversation_id: &str,
    conn: &mut PoolConnection<Sqlite>,
) -> Result<()> {
    sqlx::query!(
        "UPDATE user SET conversation_id = ? WHERE id = ?",
        conversation_id,
        user_id,
    )
    .execute(&mut **conn)
    .await?;

    Ok(())
}

pub async fn get_conversation_by_id(
    id: &str,
    conn: &mut PoolConnection<Sqlite>,
) -> Result<Option<Option<String>>> {
    let result = sqlx::query_scalar!("SELECT conversation_id FROM user WHERE id = ?", id)
        .fetch_optional(&mut **conn)
        .await?;

    Ok(result)
}
