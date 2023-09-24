use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::types::{private, FromSql, Type};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::{Pool, Transaction};
use fallible_iterator::FallibleIterator;
use postgres_protocol::types;
use sea_query::{
    Asterisk, Expr, Func, Iden, OnConflict, PostgresQueryBuilder, Query,
    SimpleExpr,
};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::db::{get_type_from_oid, ArrayAgg, DbError};

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "shop_customizations")]
pub enum ShopCustomizationIden {
    Table,
    ShopId,
    UserId,
    CreatedAt,
    UpdatedAt,
    LogoImageUrlPath,
    BannerImageUrlPath,
    HeaderBackgroundColorLight,
    HeaderBackgroundColorDark,
    HeaderContentColorLight,
    HeaderContentColorDark,
    SecondaryBackgroundColorLight,
    SecondaryBackgroundColorDark,
    SecondaryContentColorLight,
    SecondaryContentColorDark,
}

#[derive(Debug, Clone)]
pub struct ShopCustomization {
    pub shop_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub logo_image_url_path: Option<String>,
    pub banner_image_url_path: Option<String>,
    pub header_background_color_light: Option<String>,
    pub header_background_color_dark: Option<String>,
    pub header_content_color_light: Option<String>,
    pub header_content_color_dark: Option<String>,
    pub secondary_background_color_light: Option<String>,
    pub secondary_background_color_dark: Option<String>,
    pub secondary_content_color_light: Option<String>,
    pub secondary_content_color_dark: Option<String>,
}

impl ShopCustomization {
    const PUT_COLUMNS: [ShopCustomizationIden; 10] = [
        ShopCustomizationIden::ShopId,
        ShopCustomizationIden::UserId,
        ShopCustomizationIden::HeaderBackgroundColorLight,
        ShopCustomizationIden::HeaderBackgroundColorDark,
        ShopCustomizationIden::HeaderContentColorLight,
        ShopCustomizationIden::HeaderContentColorDark,
        ShopCustomizationIden::SecondaryBackgroundColorLight,
        ShopCustomizationIden::SecondaryBackgroundColorDark,
        ShopCustomizationIden::SecondaryContentColorLight,
        ShopCustomizationIden::SecondaryContentColorDark,
    ];

    pub async fn put(
        pool: &Pool,
        shop_id: &Uuid,
        user_id: &String,
        header_background_color_light: Option<String>,
        header_background_color_dark: Option<String>,
        header_content_color_light: Option<String>,
        header_content_color_dark: Option<String>,
        secondary_background_color_light: Option<String>,
        secondary_background_color_dark: Option<String>,
        secondary_content_color_light: Option<String>,
        secondary_content_color_dark: Option<String>,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(ShopCustomizationIden::Table)
            .columns(Self::PUT_COLUMNS)
            .values([
                (*shop_id).into(),
                user_id.into(),
                header_background_color_light.into(),
                header_background_color_dark.into(),
                header_content_color_light.into(),
                header_content_color_dark.into(),
                secondary_background_color_light.into(),
                secondary_background_color_dark.into(),
                secondary_content_color_light.into(),
                secondary_content_color_dark.into(),
            ])?
            .on_conflict(
                OnConflict::column(ShopCustomizationIden::ShopId)
                    .update_columns(Self::PUT_COLUMNS)
                    .to_owned(),
            )
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        shop_id: &Uuid,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(ShopCustomizationIden::Table)
            .and_where(Expr::col(ShopCustomizationIden::ShopId).eq(*shop_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn update_logo_image_url_path<'a>(
        transaction: &Transaction<'a>,
        shop_id: &Uuid,
        user_id: &String,
        logo_image_url_path: Option<String>,
    ) -> Result<Self, DbError> {
        let (sql, values) = Query::update()
            .table(ShopCustomizationIden::Table)
            .value(ShopCustomizationIden::LogoImageUrlPath, logo_image_url_path)
            .and_where(Expr::col(ShopCustomizationIden::UserId).eq(user_id))
            .and_where(Expr::col(ShopCustomizationIden::ShopId).eq(*shop_id))
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = transaction
            .query_one(sql.as_str(), &values.as_params())
            .await?;

        Ok(Self::from(row))
    }

    pub async fn update_banner_image_url_path<'a>(
        transaction: &Transaction<'a>,
        shop_id: &Uuid,
        user_id: &String,
        banner_image_url_path: Option<String>,
    ) -> Result<Self, DbError> {
        let (sql, values) = Query::update()
            .table(ShopCustomizationIden::Table)
            .value(
                ShopCustomizationIden::BannerImageUrlPath,
                banner_image_url_path,
            )
            .and_where(Expr::col(ShopCustomizationIden::UserId).eq(user_id))
            .and_where(Expr::col(ShopCustomizationIden::ShopId).eq(*shop_id))
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = transaction
            .query_one(sql.as_str(), &values.as_params())
            .await?;

        Ok(Self::from(row))
    }

    pub async fn delete<'a>(
        transaction: &Transaction<'a>,
        shop_id: &Uuid,
        user_id: &String,
    ) -> Result<(), DbError> {
        let (sql, values) = Query::delete()
            .from_table(ShopCustomizationIden::Table)
            .and_where(Expr::col(ShopCustomizationIden::UserId).eq(user_id))
            .and_where(Expr::col(ShopCustomizationIden::ShopId).eq(*shop_id))
            .build_postgres(PostgresQueryBuilder);

        transaction.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for ShopCustomization {
    fn from(row: &Row) -> Self {
        Self {
            shop_id: row
                .get(ShopCustomizationIden::ShopId.to_string().as_str()),
            user_id: row
                .get(ShopCustomizationIden::UserId.to_string().as_str()),
            created_at: row
                .get(ShopCustomizationIden::CreatedAt.to_string().as_str()),
            updated_at: row
                .get(ShopCustomizationIden::UpdatedAt.to_string().as_str()),
            logo_image_url_path: row.get(
                ShopCustomizationIden::LogoImageUrlPath.to_string().as_str(),
            ),
            banner_image_url_path: row.get(
                ShopCustomizationIden::BannerImageUrlPath
                    .to_string()
                    .as_str(),
            ),
            header_background_color_light: row.get(
                ShopCustomizationIden::HeaderBackgroundColorLight
                    .to_string()
                    .as_str(),
            ),
            header_background_color_dark: row.get(
                ShopCustomizationIden::HeaderBackgroundColorDark
                    .to_string()
                    .as_str(),
            ),
            header_content_color_light: row.get(
                ShopCustomizationIden::HeaderContentColorLight
                    .to_string()
                    .as_str(),
            ),
            header_content_color_dark: row.get(
                ShopCustomizationIden::HeaderContentColorDark
                    .to_string()
                    .as_str(),
            ),
            secondary_background_color_light: row.get(
                ShopCustomizationIden::SecondaryBackgroundColorLight
                    .to_string()
                    .as_str(),
            ),
            secondary_background_color_dark: row.get(
                ShopCustomizationIden::SecondaryBackgroundColorDark
                    .to_string()
                    .as_str(),
            ),
            secondary_content_color_light: row.get(
                ShopCustomizationIden::SecondaryContentColorLight
                    .to_string()
                    .as_str(),
            ),
            secondary_content_color_dark: row.get(
                ShopCustomizationIden::SecondaryContentColorDark
                    .to_string()
                    .as_str(),
            ),
        }
    }
}

impl From<Row> for ShopCustomization {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

#[derive(Debug, Clone)]
pub struct ShopCustomizationAsRel {
    pub logo_image_url_path: Option<String>,
    pub banner_image_url_path: Option<String>,
    pub header_background_color_light: Option<String>,
    pub header_background_color_dark: Option<String>,
    pub header_content_color_light: Option<String>,
    pub header_content_color_dark: Option<String>,
    pub secondary_background_color_light: Option<String>,
    pub secondary_background_color_dark: Option<String>,
    pub secondary_content_color_light: Option<String>,
    pub secondary_content_color_dark: Option<String>,
}

impl ShopCustomizationAsRel {
    pub fn get_agg() -> SimpleExpr {
        Func::cust(ArrayAgg)
            .args([Expr::tuple([
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::LogoImageUrlPath,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::BannerImageUrlPath,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::HeaderBackgroundColorLight,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::HeaderBackgroundColorDark,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::HeaderContentColorLight,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::HeaderContentColorDark,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::SecondaryBackgroundColorLight,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::SecondaryBackgroundColorDark,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::SecondaryContentColorLight,
                ))
                .into(),
                Expr::col((
                    ShopCustomizationIden::Table,
                    ShopCustomizationIden::SecondaryContentColorDark,
                ))
                .into(),
            ])
            .into()])
            .into()
    }
}

impl<'a> FromSql<'a> for ShopCustomizationAsRel {
    fn accepts(ty: &deadpool_postgres::tokio_postgres::types::Type) -> bool {
        matches!(*ty, Type::RECORD)
    }

    fn from_sql(
        _: &Type,
        mut raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        private::read_be_i32(&mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let logo_image_url_path: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let banner_image_url_path: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let header_background_color_light: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let header_background_color_dark: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let header_content_color_light: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let header_content_color_dark: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let secondary_background_color_light: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let secondary_background_color_dark: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let secondary_content_color_light: Option<String> =
            private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let secondary_content_color_dark: Option<String> =
            private::read_value(&ty, &mut raw)?;

        Ok(Self {
            logo_image_url_path,
            banner_image_url_path,
            header_background_color_light,
            header_background_color_dark,
            header_content_color_light,
            header_content_color_dark,
            secondary_background_color_light,
            secondary_background_color_dark,
            secondary_content_color_light,
            secondary_content_color_dark,
        })
    }
}

#[derive(Debug)]
pub struct ShopCustomizationAsRelVec(pub Vec<ShopCustomizationAsRel>);

impl<'a> FromSql<'a> for ShopCustomizationAsRelVec {
    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::RECORD_ARRAY => true,
            _ => {
                tracing::log::error!("[ShopCustomizationAsRelVec::<FromSql>::accepts]: postgres type {:?} not implemented", ty);
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
                    Ok(ShopCustomizationAsRel::from_sql_nullable(
                        &Type::RECORD,
                        v,
                    )
                    .ok())
                })
                .collect()?,
        ))
    }
}
