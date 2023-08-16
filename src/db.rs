use std::ops::DerefMut;

use deadpool_postgres::{
    tokio_postgres::NoTls, Config, CreatePoolError, Pool, PoolError, Runtime,
    SslMode,
};

use crate::get_env_var;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

#[derive(Debug)]
pub enum DbError {
    TokioPostgres(deadpool_postgres::tokio_postgres::Error),
    Pool(PoolError),
    CreatePool(CreatePoolError),
    SeaQuery(sea_query::error::Error),
}

impl From<deadpool_postgres::tokio_postgres::Error> for DbError {
    fn from(err: deadpool_postgres::tokio_postgres::Error) -> Self {
        Self::TokioPostgres(err)
    }
}

impl From<PoolError> for DbError {
    fn from(err: PoolError) -> Self {
        Self::Pool(err)
    }
}

impl From<CreatePoolError> for DbError {
    fn from(err: CreatePoolError) -> Self {
        Self::CreatePool(err)
    }
}

impl From<sea_query::error::Error> for DbError {
    fn from(err: sea_query::error::Error) -> Self {
        Self::SeaQuery(err)
    }
}

pub fn init_db_pool() -> Result<Pool, CreatePoolError> {
    let mut config = Config::new();
    config.host = Some(get_env_var("DB_HOST"));
    config.port = Some(get_env_var("DB_PORT").parse().unwrap());
    config.user = Some(get_env_var("DB_USER"));
    config.password = Some(get_env_var("DB_PASSWORD"));
    config.dbname = Some(get_env_var("DB_DBNAME"));

    config.ssl_mode = Some(SslMode::Prefer);

    config.create_pool(Some(Runtime::Tokio1), NoTls)
}

pub async fn migrate(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;

    embedded::migrations::runner()
        .run_async(client.deref_mut().deref_mut())
        .await?;

    Ok(())
}
