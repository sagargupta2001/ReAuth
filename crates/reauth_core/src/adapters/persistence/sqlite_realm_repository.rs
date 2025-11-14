use crate::adapters::persistence::connection::Database;
use crate::{
    domain::realm::Realm,
    error::{Error, Result},
    ports::realm_repository::RealmRepository,
};
use async_trait::async_trait;
use uuid::Uuid;

pub struct SqliteRealmRepository {
    pool: Database,
}
impl SqliteRealmRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RealmRepository for SqliteRealmRepository {
    async fn create(&self, realm: &Realm) -> Result<()> {
        sqlx::query("INSERT INTO realms (id, name, access_token_ttl_secs, refresh_token_ttl_secs) VALUES (?, ?, ?, ?)")
            .bind(realm.id.to_string())
            .bind(&realm.name)
            .bind(realm.access_token_ttl_secs)
            .bind(realm.refresh_token_ttl_secs)
            .execute(&*self.pool).await.map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Realm>> {
        Ok(sqlx::query_as("SELECT * FROM realms WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?)
    }
    async fn find_by_name(&self, name: &str) -> Result<Option<Realm>> {
        Ok(sqlx::query_as("SELECT * FROM realms WHERE name = ?")
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?)
    }
}
