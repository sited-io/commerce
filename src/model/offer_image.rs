use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::types::{private, FromSql, Type};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::{Pool, Transaction};
use fallible_iterator::FallibleIterator;
use postgres_protocol::types;
use sea_query::{
    Asterisk, Expr, Func, Iden, PostgresQueryBuilder, Query, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::db::{get_type_from_oid, ArrayAgg, DbError};

#[derive(Iden)]
#[iden(rename = "offer_images")]
pub enum OfferImageIden {
    Table,
    OfferImageId,
    OfferId,
    UserId,
    CreatedAt,
    UpdatedAt,
    ImageUrlPath,
    Ordering,
}

#[derive(Debug, Clone)]
pub struct OfferImage {
    pub offer_image_id: Uuid,
    pub offer_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub image_url_path: String,
    pub ordering: i64,
}

impl OfferImage {
    pub async fn create<'a>(
        transaction: &Transaction<'a>,
        offer_image_id: &Uuid,
        offer_id: &Uuid,
        user_id: &String,
        image_url_path: &String,
        ordering: i64,
    ) -> Result<Self, DbError> {
        let (sql, values) = Query::insert()
            .into_table(OfferImageIden::Table)
            .columns([
                OfferImageIden::OfferImageId,
                OfferImageIden::OfferId,
                OfferImageIden::UserId,
                OfferImageIden::ImageUrlPath,
                OfferImageIden::Ordering,
            ])
            .values([
                (*offer_image_id).into(),
                (*offer_id).into(),
                user_id.into(),
                image_url_path.into(),
                ordering.into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = transaction
            .query_one(sql.as_str(), &values.as_params())
            .await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        offer_image_id: &Uuid,
        user_id: Option<&String>,
    ) -> Result<Option<Self>, DbError> {
        let client = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::select();

            query
                .column(Asterisk)
                .from(OfferImageIden::Table)
                .and_where(
                    Expr::col(OfferImageIden::OfferImageId).eq(*offer_image_id),
                );

            if let Some(user_id) = user_id {
                query.and_where(Expr::col(OfferImageIden::UserId).eq(user_id));
            }

            query.build_postgres(PostgresQueryBuilder)
        };

        let row = client.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn delete<'a>(
        transaction: &Transaction<'a>,
        user_id: &String,
        offer_image_id: &Uuid,
    ) -> Result<(), DbError> {
        let (sql, values) = Query::delete()
            .from_table(OfferImageIden::Table)
            .and_where(Expr::col(OfferImageIden::UserId).eq(user_id))
            .and_where(
                Expr::col(OfferImageIden::OfferImageId).eq(*offer_image_id),
            )
            .build_postgres(PostgresQueryBuilder);

        transaction
            .execute(sql.as_str(), &values.as_params())
            .await?;

        Ok(())
    }
}

impl From<&Row> for OfferImage {
    fn from(row: &Row) -> Self {
        Self {
            offer_image_id: row
                .get(OfferImageIden::OfferImageId.to_string().as_str()),
            offer_id: row.get(OfferImageIden::OfferId.to_string().as_str()),
            user_id: row.get(OfferImageIden::UserId.to_string().as_str()),
            created_at: row.get(OfferImageIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(OfferImageIden::UpdatedAt.to_string().as_str()),
            image_url_path: row
                .get(OfferImageIden::ImageUrlPath.to_string().as_str()),
            ordering: row.get(OfferImageIden::Ordering.to_string().as_str()),
        }
    }
}

impl From<Row> for OfferImage {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

#[derive(Debug, Clone)]
pub struct OfferImageAsRel {
    pub offer_image_id: Uuid,
    pub image_url_path: String,
    pub ordering: i64,
}

impl OfferImageAsRel {
    pub fn get_agg() -> SimpleExpr {
        Func::cust(ArrayAgg)
            .args([Expr::tuple([
                Expr::col((
                    OfferImageIden::Table,
                    OfferImageIden::OfferImageId,
                ))
                .into(),
                Expr::col((
                    OfferImageIden::Table,
                    OfferImageIden::ImageUrlPath,
                ))
                .into(),
                Expr::col((OfferImageIden::Table, OfferImageIden::Ordering))
                    .into(),
            ])
            .into()])
            .into()
    }
}

impl<'a> FromSql<'a> for OfferImageAsRel {
    fn accepts(ty: &deadpool_postgres::tokio_postgres::types::Type) -> bool {
        match *ty {
            Type::RECORD => true,
            _ => {
                tracing::log::error!("OfferImageAsRel FromSql accepts: postgres type {:?} not implemented", ty);
                false
            }
        }
    }

    fn from_sql(
        _: &deadpool_postgres::tokio_postgres::types::Type,
        mut raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        private::read_be_i32(&mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Uuid>(oid)?;
        let offer_image_id: Uuid = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let image_url_path: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<i64>(oid)?;
        let ordering: i64 = private::read_value(&ty, &mut raw)?;

        Ok(Self {
            offer_image_id,
            image_url_path,
            ordering,
        })
    }
}

pub struct OfferImageAsRelVec(pub Vec<OfferImageAsRel>);

impl<'a> FromSql<'a> for OfferImageAsRelVec {
    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::RECORD_ARRAY => true,
            _ => {
                tracing::log::error!("OfferImageAsRelVec FromSql accepts: postgres type {:?} not implemented", ty);
                false
            }
        }
    }

    fn from_sql(
        _: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let array = types::array_from_sql(raw)?;

        if array.dimensions().count()? > 1 {
            return Err("array contains too many dimensions".into());
        }

        Ok(Self(
            array
                .values()
                .filter_map(|v| {
                    Ok(OfferImageAsRel::from_sql_nullable(&Type::RECORD, v)
                        .ok())
                })
                .collect()?,
        ))
    }
}
