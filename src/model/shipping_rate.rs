use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::{Pool, Transaction};
use sea_query::{
    all, Asterisk, Expr, Iden, OnConflict, PostgresQueryBuilder, Query,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::db::DbError;

#[derive(Iden)]
#[iden(rename = "shipping_rates")]
pub enum ShippingRateIden {
    Table,
    ShippingRateId,
    OfferId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Amount,
    Currency,
    AllCountries,
    SpecificCountries,
}

#[derive(Debug, Clone)]
pub struct ShippingRate {
    pub shipping_rate_id: Uuid,
    pub offer_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub amount: u32,
    pub currency: String,
    pub all_countries: bool,
    pub specific_countries: Option<String>,
}

impl ShippingRate {
    const PUT_COLUMNS: [ShippingRateIden; 6] = [
        ShippingRateIden::OfferId,
        ShippingRateIden::UserId,
        ShippingRateIden::Amount,
        ShippingRateIden::Currency,
        ShippingRateIden::AllCountries,
        ShippingRateIden::SpecificCountries,
    ];

    pub async fn put(
        pool: &Pool,
        offer_id: &Uuid,
        user_id: &String,
        amount: u32,
        currency: &str,
        all_countries: bool,
        specific_countries: Option<String>,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(ShippingRateIden::Table)
            .columns(Self::PUT_COLUMNS)
            .values([
                (*offer_id).into(),
                user_id.into(),
                i64::from(amount).into(),
                currency.into(),
                all_countries.into(),
                specific_countries.into(),
            ])?
            .on_conflict(
                OnConflict::columns([
                    ShippingRateIden::OfferId,
                    ShippingRateIden::UserId,
                    ShippingRateIden::Currency,
                ])
                .update_columns(Self::PUT_COLUMNS)
                .to_owned(),
            )
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get_by_offer_id(
        pool: &Pool,
        offer_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(ShippingRateIden::Table)
            .and_where(Expr::col(ShippingRateIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn delete(
        pool: &Pool,
        shipping_rate_id: &Uuid,
        user_id: &String,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(ShippingRateIden::Table)
            .cond_where(all![
                Expr::col(ShippingRateIden::ShippingRateId)
                    .eq(*shipping_rate_id),
                Expr::col(ShippingRateIden::UserId).eq(user_id)
            ])
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let shipping_rate = conn
            .query_one(sql.as_str(), &values.as_params())
            .await?
            .into();

        Ok(shipping_rate)
    }

    pub async fn delete_all<'a>(
        transaction: &Transaction<'a>,
        user_id: &String,
        offer_id: &Uuid,
    ) -> Result<(), DbError> {
        let (sql, values) = Query::delete()
            .from_table(ShippingRateIden::Table)
            .and_where(Expr::col(ShippingRateIden::UserId).eq(user_id))
            .and_where(Expr::col(ShippingRateIden::OfferId).eq(*offer_id))
            .build_postgres(PostgresQueryBuilder);

        transaction
            .execute(sql.as_str(), &values.as_params())
            .await?;

        Ok(())
    }
}

impl From<&Row> for ShippingRate {
    fn from(row: &Row) -> Self {
        Self {
            shipping_rate_id: row
                .get(ShippingRateIden::ShippingRateId.to_string().as_str()),
            offer_id: row.get(ShippingRateIden::OfferId.to_string().as_str()),
            user_id: row.get(ShippingRateIden::UserId.to_string().as_str()),
            created_at: row
                .get(ShippingRateIden::CreatedAt.to_string().as_str()),
            updated_at: row
                .get(ShippingRateIden::UpdatedAt.to_string().as_str()),
            amount: u32::try_from(row.get::<&str, i64>(
                ShippingRateIden::Amount.to_string().as_str(),
            ))
            .unwrap(),
            currency: row.get(ShippingRateIden::Currency.to_string().as_str()),
            all_countries: row
                .get(ShippingRateIden::AllCountries.to_string().as_str()),
            specific_countries: row
                .get(ShippingRateIden::SpecificCountries.to_string().as_str()),
        }
    }
}

impl From<Row> for ShippingRate {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}
