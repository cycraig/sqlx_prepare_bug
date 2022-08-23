use rand::distributions::{Alphanumeric, DistString};
use sqlx::postgres::PgPool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database).await?;

    // Insert a new, random row.
    let name: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let record = sqlx::query!(
        r#"
INSERT INTO table_c ( name_c )
VALUES ( $1 )
RETURNING id_c
        "#,
        name
    )
    .fetch_one(&pool)
    .await?;
    let id = record.id_c;

    println!("Inserted ({id}, {name})");
    Ok(())
}
