use sqlx::{pool::PoolConnection, postgres::PgPool, Error as SqlxError, PgConnection, Transaction};
use std::sync::Arc;

pub struct PgStore {
    pool: Arc<PgPool>,
}

impl PgStore {
    pub fn new(pool: Arc<sqlx::PgPool>) -> Self {
        PgStore { pool: pool }
    }

    pub async fn get_tx(&self) -> Result<Transaction<PoolConnection<PgConnection>>, SqlxError> {
        self.pool.begin().await
    }
}
