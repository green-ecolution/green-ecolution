use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use domain::{
    Id, IdSliceExt, RawId, RepositoryError,
    authorization::Visibility,
    organization::Organization,
    sensor::SensorId,
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination, SearchPage},
        provenance::Provenance,
    },
};
use waste_domain::{
    fill::FillMeasurement,
    values::{BinDepth, BinVolume, WasteBinNumber},
    waste_bin::{
        FillFilter, WasteBin, WasteBinDraft, WasteBinReader, WasteBinSearchQuery, WasteBinSnapshot,
        WasteBinView, WasteBinWriter,
    },
};

pub struct PgWasteBinRepository {
    pool: PgPool,
}

impl PgWasteBinRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Visible orgs narrowed by the requested one; an invisible request matches
/// nothing instead of failing, as in the tree list.
fn org_scope_ids(visible: &Visibility, requested: Option<Id<Organization>>) -> Option<Vec<RawId>> {
    let requested = requested.map(|id| id.value());
    match (visible.clone().into_raw_ids(), requested) {
        (None, None) => None,
        (None, Some(org)) => Some(vec![org]),
        (Some(visible), None) => Some(visible),
        (Some(visible), Some(org)) => Some(visible.into_iter().filter(|id| *id == org).collect()),
    }
}

/// `ILIKE` substring pattern with the wildcards of the input taken literally.
fn contains_pattern(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    out.push('%');
    for ch in input.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('%');
    out
}

struct WasteBinViewRow {
    id: RawId,
    number: String,
    latitude: f64,
    longitude: f64,
    volume_liters: f64,
    depth_cm: i32,
    organization_id: RawId,
    provider: Option<String>,
    additional_informations: Option<Value>,
    fill_fraction: Option<f64>,
    fill_measured_at: Option<DateTime<Utc>>,
    last_emptied_at: Option<DateTime<Utc>>,
    sensor_id: Option<String>,
    updated_at: NaiveDateTime,
}

impl TryFrom<WasteBinViewRow> for WasteBinView {
    type Error = RepositoryError;

    fn try_from(row: WasteBinViewRow) -> Result<Self, Self::Error> {
        let id = Id::<WasteBin>::new(row.id);
        let created_at = id.created_at().ok_or_else(|| {
            RepositoryError::DataIntegrity("waste_bins.id is not a uuid v7".into())
        })?;
        let fill = match (row.fill_fraction, row.fill_measured_at) {
            (Some(fraction), Some(measured_at)) => Some(FillMeasurement {
                fraction,
                measured_at,
            }),
            _ => None,
        };
        Ok(Self {
            id,
            number: WasteBinNumber::new(row.number)?,
            coordinate: Coordinate::new(row.latitude, row.longitude)?,
            volume: BinVolume::new(row.volume_liters)?,
            depth: BinDepth::new(i64::from(row.depth_cm))?,
            organization_id: Id::new(row.organization_id),
            provenance: Provenance::reconstitute(row.provider, row.additional_informations),
            fill,
            last_emptied_at: row.last_emptied_at,
            sensor_id: row.sensor_id.map(SensorId::new).transpose()?,
            created_at,
            updated_at: row.updated_at.and_utc(),
        })
    }
}

fn into_views(rows: Vec<WasteBinViewRow>) -> Result<Vec<WasteBinView>, RepositoryError> {
    rows.into_iter().map(TryInto::try_into).collect()
}

fn reconstitute(snap: WasteBinSnapshot) -> Result<WasteBin, RepositoryError> {
    Ok(WasteBin::reconstitute(snap)?)
}

#[async_trait::async_trait]
impl WasteBinReader for PgWasteBinRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<WasteBin>) -> Result<WasteBin, RepositoryError> {
        let snap = sqlx::query_as!(
            WasteBinSnapshot,
            r#"SELECT id, number, latitude, longitude, volume_liters, depth_cm, organization_id,
                      provider, additional_informations,
                      fill_fraction, fill_measured_at, last_emptied_at
            FROM waste_bins WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        reconstitute(snap)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[Id<WasteBin>]) -> Result<Vec<WasteBin>, RepositoryError> {
        let snaps = sqlx::query_as!(
            WasteBinSnapshot,
            r#"SELECT id, number, latitude, longitude, volume_liters, depth_cm, organization_id,
                      provider, additional_informations,
                      fill_fraction, fill_measured_at, last_emptied_at
            FROM waste_bins WHERE id = ANY($1::uuid[])"#,
            &ids.to_values()
        )
        .fetch_all(&self.pool)
        .await?;

        snaps.into_iter().map(reconstitute).collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(&self, id: Id<WasteBin>) -> Result<WasteBinView, RepositoryError> {
        let row = sqlx::query_as!(
            WasteBinViewRow,
            r#"SELECT b.id, b.number, b.latitude, b.longitude, b.volume_liters, b.depth_cm,
                      b.organization_id, b.provider, b.additional_informations,
                      b.fill_fraction, b.fill_measured_at, b.last_emptied_at,
                      sa.sensor_id AS "sensor_id?", b.updated_at
            FROM waste_bins b
            LEFT JOIN sensor_attachments sa
                   ON sa.module_key = 'waste' AND sa.target_kind = 'bin' AND sa.target_id = b.id
            WHERE b.id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        row.try_into()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_ids(
        &self,
        ids: &[Id<WasteBin>],
    ) -> Result<Vec<WasteBinView>, RepositoryError> {
        let rows = sqlx::query_as!(
            WasteBinViewRow,
            r#"SELECT b.id, b.number, b.latitude, b.longitude, b.volume_liters, b.depth_cm,
                      b.organization_id, b.provider, b.additional_informations,
                      b.fill_fraction, b.fill_measured_at, b.last_emptied_at,
                      sa.sensor_id AS "sensor_id?", b.updated_at
            FROM waste_bins b
            LEFT JOIN sensor_attachments sa
                   ON sa.module_key = 'waste' AND sa.target_kind = 'bin' AND sa.target_id = b.id
            WHERE b.id = ANY($1::uuid[])
            ORDER BY b.number, b.id"#,
            &ids.to_values()
        )
        .fetch_all(&self.pool)
        .await?;

        into_views(rows)
    }

    /// Organization and visibility scope the unfiltered total; number and
    /// fill narrow it.
    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_search(
        &self,
        query: WasteBinSearchQuery,
        pagination: Pagination,
        visibility: &Visibility,
    ) -> Result<SearchPage<WasteBinView>, RepositoryError> {
        let scope = org_scope_ids(visibility, query.organization_id);
        let number = query
            .number
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(contains_pattern);
        let fill = query.fill.unwrap_or_default();
        let FillFilter {
            min_fraction,
            max_fraction,
            measured_after,
            stale_before,
            require_measurement,
            not_emptied_since_measurement,
            emptied_after_measurement,
        } = fill;
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let rows = sqlx::query_as!(
            WasteBinViewRow,
            r#"SELECT b.id, b.number, b.latitude, b.longitude, b.volume_liters, b.depth_cm,
                      b.organization_id, b.provider, b.additional_informations,
                      b.fill_fraction, b.fill_measured_at, b.last_emptied_at,
                      sa.sensor_id AS "sensor_id?", b.updated_at
            FROM waste_bins b
            LEFT JOIN sensor_attachments sa
                   ON sa.module_key = 'waste' AND sa.target_kind = 'bin' AND sa.target_id = b.id
            WHERE ($1::uuid[] IS NULL OR b.organization_id = ANY($1))
              AND ($2::text IS NULL OR b.number ILIKE $2 ESCAPE '\')
              AND ($3::float8 IS NULL OR b.fill_fraction >= $3)
              AND ($4::float8 IS NULL OR b.fill_fraction < $4)
              AND ($5::timestamptz IS NULL OR b.fill_measured_at >= $5)
              AND ($6::timestamptz IS NULL OR b.fill_measured_at IS NULL OR b.fill_measured_at < $6)
              AND (NOT $7::bool OR b.fill_measured_at IS NOT NULL)
              AND (NOT $8::bool OR b.last_emptied_at IS NULL OR b.last_emptied_at <= b.fill_measured_at)
              AND (NOT $9::bool OR b.last_emptied_at > b.fill_measured_at)
            ORDER BY b.number, b.id
            LIMIT $10 OFFSET $11"#,
            scope.as_deref(),
            number.as_deref(),
            min_fraction,
            max_fraction,
            measured_after,
            stale_before,
            require_measurement,
            not_emptied_since_measurement,
            emptied_after_measurement,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM waste_bins b
            WHERE ($1::uuid[] IS NULL OR b.organization_id = ANY($1))
              AND ($2::text IS NULL OR b.number ILIKE $2 ESCAPE '\')
              AND ($3::float8 IS NULL OR b.fill_fraction >= $3)
              AND ($4::float8 IS NULL OR b.fill_fraction < $4)
              AND ($5::timestamptz IS NULL OR b.fill_measured_at >= $5)
              AND ($6::timestamptz IS NULL OR b.fill_measured_at IS NULL OR b.fill_measured_at < $6)
              AND (NOT $7::bool OR b.fill_measured_at IS NOT NULL)
              AND (NOT $8::bool OR b.last_emptied_at IS NULL OR b.last_emptied_at <= b.fill_measured_at)
              AND (NOT $9::bool OR b.last_emptied_at > b.fill_measured_at)"#,
            scope.as_deref(),
            number.as_deref(),
            min_fraction,
            max_fraction,
            measured_after,
            stale_before,
            require_measurement,
            not_emptied_since_measurement,
            emptied_after_measurement,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_unfiltered = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM waste_bins
            WHERE ($1::uuid[] IS NULL OR organization_id = ANY($1))"#,
            scope.as_deref(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SearchPage {
            page: Page {
                items: into_views(rows)?,
                total: total.max(0) as u64,
            },
            total_unfiltered: total_unfiltered.max(0) as u64,
        })
    }
}

#[async_trait::async_trait]
impl WasteBinWriter for PgWasteBinRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: WasteBinDraft) -> Result<WasteBin, RepositoryError> {
        let id = Id::<WasteBin>::new_v7();
        sqlx::query!(
            r#"INSERT INTO waste_bins (id, number, latitude, longitude, geometry,
                                       volume_liters, depth_cm, organization_id,
                                       provider, additional_informations)
            VALUES ($1, $2, $3, $4, ST_SetSRID(ST_MakePoint($4, $3), 4326),
                    $5, $6, $7, $8, $9)"#,
            id.value(),
            draft.number.as_str(),
            draft.coordinate.latitude(),
            draft.coordinate.longitude(),
            draft.volume.liters(),
            draft.depth.centimeters() as i32,
            draft.organization_id.value(),
            draft.provenance.provider().map(|p| p.as_str().to_owned()),
            draft.provenance.additional_info().cloned(),
        )
        .execute(&self.pool)
        .await?;

        self.by_id(id).await
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, bin: &WasteBin) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"UPDATE waste_bins SET
                number = $2,
                latitude = $3,
                longitude = $4,
                geometry = ST_SetSRID(ST_MakePoint($4, $3), 4326),
                volume_liters = $5,
                depth_cm = $6,
                organization_id = $7,
                provider = $8,
                additional_informations = $9
            WHERE id = $1"#,
            bin.id.value(),
            bin.number.as_str(),
            bin.coordinate.latitude(),
            bin.coordinate.longitude(),
            bin.volume.liters(),
            bin.depth.centimeters() as i32,
            bin.organization_id().value(),
            bin.provenance().provider().map(|p| p.as_str().to_owned()),
            bin.provenance().additional_info().cloned(),
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_fill_level(&self, bin: &WasteBin) -> Result<(), RepositoryError> {
        let Some(fill) = bin.fill() else {
            return Ok(());
        };
        sqlx::query!(
            r#"UPDATE waste_bins SET fill_fraction = $2, fill_measured_at = $3
            WHERE id = $1 AND (fill_measured_at IS NULL OR fill_measured_at < $3)"#,
            bin.id.value(),
            fill.fraction,
            fill.measured_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_emptied(&self, bin: &WasteBin) -> Result<(), RepositoryError> {
        let Some(emptied_at) = bin.last_emptied_at() else {
            return Ok(());
        };
        sqlx::query!(
            r#"UPDATE waste_bins SET last_emptied_at = $2
            WHERE id = $1 AND (last_emptied_at IS NULL OR last_emptied_at < $2)"#,
            bin.id.value(),
            emptied_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<WasteBin>) -> Result<(), RepositoryError> {
        let result = sqlx::query!("DELETE FROM waste_bins WHERE id = $1", id.value())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn org(n: u128) -> Id<Organization> {
        Id::new(uuid::Uuid::from_u128(n))
    }

    fn only(ids: &[u128]) -> Visibility {
        Visibility::Only(ids.iter().map(|n| org(*n)).collect::<BTreeSet<_>>())
    }

    #[test]
    fn unrestricted_without_request_does_not_filter() {
        assert_eq!(org_scope_ids(&Visibility::Unrestricted, None), None);
    }

    #[test]
    fn request_narrows_the_visible_orgs() {
        assert_eq!(
            org_scope_ids(&only(&[1, 2]), Some(org(2))),
            Some(vec![org(2).value()])
        );
    }

    #[test]
    fn requesting_an_invisible_org_matches_nothing() {
        assert_eq!(org_scope_ids(&only(&[1, 2]), Some(org(9))), Some(vec![]));
    }

    #[test]
    fn wildcards_in_the_search_are_literal() {
        assert_eq!(contains_pattern("A_1%"), r"%A\_1\%%");
        assert_eq!(contains_pattern(r"a\b"), r"%a\\b%");
    }
}
