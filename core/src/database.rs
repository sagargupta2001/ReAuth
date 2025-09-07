use std::sync::Arc;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

pub type Database = Arc<Surreal<Db>>;

pub async fn init_db() -> anyhow::Result<Database> {
    let db = Surreal::new::<RocksDb>("db").await?;
    db.use_ns("test").use_db("test").await?;
    Ok(Arc::new(db))
}
