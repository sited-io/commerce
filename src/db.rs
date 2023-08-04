use deadpool_postgres::{
    tokio_postgres::NoTls, Config, CreatePoolError, Pool, PoolError, Runtime,
    SslMode,
};
use sea_query::{ColumnDef, PostgresQueryBuilder, Table};

use crate::{get_env_var, model};

pub enum DbError {
    TokioPostgres(deadpool_postgres::tokio_postgres::Error),
    Pool(PoolError),
    CreatePool(CreatePoolError),
    SeaQuery(sea_query::error::Error),
}

impl Into<String> for DbError {
    fn into(self) -> String {
        match self {
            Self::TokioPostgres(err) => err.to_string(),
            Self::Pool(err) => err.to_string(),
            Self::CreatePool(err) => err.to_string(),
            Self::SeaQuery(err) => err.to_string(),
        }
    }
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
    let client = pool.get().await?;

    let sql = [
        Table::drop()
            .table(model::ShopIden::Table)
            .if_exists()
            .build(PostgresQueryBuilder),
        Table::create()
            .table(model::ShopIden::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(model::ShopIden::ShopId)
                    .uuid()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(model::ShopIden::UserId).string().not_null())
            .col(
                ColumnDef::new(model::ShopIden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(model::ShopIden::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(ColumnDef::new(model::ShopIden::Name).string().not_null())
            .col(ColumnDef::new(model::ShopIden::Description).string())
            .build(PostgresQueryBuilder),
    ]
    .join("; ");

    client.batch_execute(&sql).await?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u64,
    pub size: u64,
}
