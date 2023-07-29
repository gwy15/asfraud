pub use sqlx::prelude::*;

pub type Pool = sqlx::SqlitePool;

pub async fn new(url: &str) -> anyhow::Result<Pool> {
    let db = Pool::connect(url).await?;
    Ok(db)
}
