use std::ops::DerefMut;

use deadpool_postgres::tokio_postgres::error::SqlState;
use deadpool_postgres::{
    tokio_postgres::NoTls, Config, CreatePoolError, Pool, PoolError, Runtime,
    SslMode,
};
use sea_query::{Expr, PgFunc, SimpleExpr};
use tonic::Status;

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

impl DbError {
    pub fn ignore_to_ts_query<T>(self, default: T) -> Result<T, Self> {
        if let Self::TokioPostgres(err) = &self {
            if let Some(err) = err.as_db_error() {
                if *err.code() == SqlState::SYNTAX_ERROR
                    && err.routine() == Some("toTSQuery")
                {
                    tracing::log::warn!("{:?}", err);
                    return Ok(default);
                }
            }
        }

        Err(self)
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

impl From<DbError> for Status {
    fn from(err: DbError) -> Self {
        match err {
            DbError::TokioPostgres(tp_err) => {
                if let Some(err) = tp_err.as_db_error() {
                    match *err.code() {
                        SqlState::UNIQUE_VIOLATION => {
                            Status::already_exists(err.message())
                        }
                        SqlState::SYNTAX_ERROR => {
                            tracing::log::error!("{err:?}");
                            Status::internal("")
                        }
                        _ => {
                            todo!("{err:?}")
                        }
                    }
                } else {
                    // tracing::log::error!("{:?}", tp_err);
                    Status::internal("")
                }
            }
            _ => {
                todo!("{err:?}")
            }
        }
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

    let runner = embedded::migrations::runner();
    runner.get_migrations();
    runner.run_async(client.deref_mut().deref_mut()).await?;

    Ok(())
}

pub fn build_simple_plain_ts_query(query: String) -> Expr {
    Expr::expr(
        PgFunc::plainto_tsquery("", None)
            .args([SimpleExpr::Value("simple".into()), query.into()]),
    )
}
