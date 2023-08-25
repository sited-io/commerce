use deadpool_postgres::tokio_postgres::error::SqlState;
use tonic::Status;

use crate::db::DbError;

pub fn db_err_to_grpc_status(db_err: DbError) -> Status {
    tracing::log::debug!("Got error from DB: {db_err:?}");

    match db_err {
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
                todo!("{:?}", tp_err)
            }
        }
        _ => {
            todo!("{db_err:?}")
        }
    }
}
