pub mod models;

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, Select};
use chrono::{DateTime, Duration, Utc};
use field::field;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::crud::play::models::PlayInDB;
use crate::crud::shared::models::{Gsi1PaginateKey, PaginateKey};
use crate::crud::station::models::{StationId, StationInDB};
use crate::crud::track::models::TrackId;
use crate::crud::track::models::{
    TrackInDB, TrackMetadataCreateInDB, TrackMetadataInDB, TrackMetadataKeys, TrackMinimalInDB,
    TrackPlayInDB,
};
use crate::crud::Context;
use crate::helpers::truncate_datetime_to_months;

const ULID_RANDOM_MAX: u128 = (1 << 80) - 1;

pub struct CRUDTrack {
    context: Arc<Context>,
}

impl CRUDTrack {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    pub async fn get_track(
        &self,
        station_id: StationId,
        track_id: TrackId,
    ) -> Result<Option<TrackInDB>> {
        let resp = self
            .context
            .db_client
            .get_item()
            .table_name(&self.context.db_table)
            .key(
                field!(pk @ TrackInDB),
                AttributeValue::S(TrackInDB::get_pk(station_id)),
            )
            .key(
                field!(sk @ TrackInDB),
                AttributeValue::S(TrackInDB::get_sk(track_id)),
            )
            .send()
            .await?;

        if let Some(item) = resp.item {
            Ok(Some(serde_dynamo::from_item(item)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_track_by_metadata(
        &self,
        station: &StationInDB,
        artist: &str,
        title: &str,
    ) -> Result<Option<TrackMetadataInDB>> {
        let resp = self
            .context
            .db_client
            .get_item()
            .table_name(&self.context.db_table)
            .key(
                field!(pk @ TrackMetadataCreateInDB),
                AttributeValue::S(TrackMetadataInDB::get_pk(station.id, artist)),
            )
            .key(
                field!(sk @ TrackMetadataCreateInDB),
                AttributeValue::S(TrackMetadataInDB::get_sk(title)),
            )
            .projection_expression(field!(track_id @ TrackMetadataInDB))
            .consistent_read(true)
            .send()
            .await?;

        if let Some(item) = resp.item {
            Ok(Some(serde_dynamo::from_item(item)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_tracks(
        &self,
        station_id: StationId,
        limit: i32,
        next_key: Option<Ulid>,
    ) -> Result<(Vec<TrackInDB>, Option<Ulid>)> {
        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("#pk = :pk AND begins_with(#sk, :sk_prefix)")
            .expression_attribute_names("#pk", field!(pk @ TrackInDB))
            .expression_attribute_names("#sk", field!(sk @ TrackInDB))
            .expression_attribute_values(":pk", AttributeValue::S(TrackInDB::get_pk(station_id)))
            .expression_attribute_values(
                ":sk_prefix",
                AttributeValue::S(TrackInDB::get_sk_prefix()),
            )
            .scan_index_forward(false)
            .select(Select::AllAttributes)
            .limit(limit);

        if let Some(next_key) = next_key {
            let track_id = next_key.into();
            query = query
                .exclusive_start_key(
                    field!(pk @ TrackInDB),
                    AttributeValue::S(TrackInDB::get_pk(station_id)),
                )
                .exclusive_start_key(
                    field!(sk @ TrackInDB),
                    AttributeValue::S(TrackInDB::get_sk(track_id)),
                );
        };

        let resp = query.send().await?;

        let next_key = if let Some(last_evaluated_key) = resp.last_evaluated_key {
            let paginate_key: PaginateKey = serde_dynamo::from_item(last_evaluated_key)?;
            Some(
                Ulid::from_string(
                    paginate_key
                        .sk
                        .strip_prefix(&TrackInDB::get_sk_prefix())
                        .expect("parse next key"),
                )
                .expect("next key into ulid"),
            )
        } else {
            None
        };

        Ok((
            serde_dynamo::from_items(resp.items.expect("query response to have items"))?,
            next_key,
        ))
    }

    pub async fn list_tracks_by_artist(
        &self,
        station_id: StationId,
        artist: &str,
        limit: i32,
        next_key: Option<&str>,
    ) -> Result<(Vec<TrackInDB>, Option<String>)> {
        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("#pk = :pk AND begins_with(#sk, :sk_prefix)")
            .expression_attribute_names("#pk", field!(pk @ TrackMetadataCreateInDB))
            .expression_attribute_names("#sk", field!(sk @ TrackMetadataCreateInDB))
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(TrackMetadataInDB::get_pk(station_id, artist)),
            )
            .expression_attribute_values(
                ":sk_prefix",
                AttributeValue::S(TrackMetadataInDB::get_sk_prefix()),
            )
            .scan_index_forward(false)
            .select(Select::SpecificAttributes)
            .projection_expression(field!(track_id @ TrackMetadataInDB))
            .limit(limit);

        if let Some(next_key) = next_key {
            query = query
                .exclusive_start_key(
                    field!(pk @ TrackMetadataCreateInDB),
                    AttributeValue::S(TrackMetadataInDB::get_pk(station_id, artist)),
                )
                .exclusive_start_key(
                    field!(sk @ TrackMetadataCreateInDB),
                    AttributeValue::S(TrackMetadataInDB::get_sk(next_key)),
                );
        };

        let resp = query.send().await?;

        let next_key = if let Some(last_evaluated_key) = resp.last_evaluated_key {
            let paginate_key: PaginateKey = serde_dynamo::from_item(last_evaluated_key)?;
            Some(
                paginate_key
                    .sk
                    .strip_prefix(&TrackMetadataInDB::get_sk_prefix())
                    .expect("parse next key")
                    .to_owned(),
            )
        } else {
            None
        };

        let track_metadatas: Vec<TrackMetadataInDB> =
            serde_dynamo::from_items(resp.items.expect("query response to have items"))?;

        let tracks = self
            .batch_get_tracks(
                station_id,
                track_metadatas.iter().map(|item| &item.track_id),
            )
            .await?;

        Ok((tracks, next_key))
    }

    pub async fn batch_get_tracks(
        &self,
        station_id: StationId,
        track_ids: impl Iterator<Item = &TrackId>,
    ) -> Result<Vec<TrackInDB>> {
        self.batch_get_tracks_internal(
            station_id,
            track_ids,
            field!(pk @ TrackInDB),
            field!(sk @ TrackInDB),
            None,
        )
        .await
    }

    pub async fn batch_get_tracks_minimal(
        &self,
        station_id: StationId,
        track_ids: impl Iterator<Item = &TrackId>,
    ) -> Result<Vec<TrackMinimalInDB>> {
        self.batch_get_tracks_internal(
            station_id,
            track_ids,
            field!(pk @ TrackInDB),
            field!(sk @ TrackInDB),
            Some(&[
                field!(id @ TrackMinimalInDB),
                field!(title @ TrackMinimalInDB),
                field!(artist @ TrackMinimalInDB),
                field!(is_song @ TrackMinimalInDB),
            ]),
        )
        .await
    }

    async fn batch_get_tracks_internal<'a, O>(
        &self,
        station_id: StationId,
        track_ids: impl Iterator<Item = &TrackId>,
        pk_key_name: &'static str,
        sk_key_name: &'static str,
        projection_fields: Option<&'static [&'static str]>,
    ) -> Result<Vec<O>>
    where
        O: Serialize + Deserialize<'a>,
    {
        let mut request_keys = KeysAndAttributes::builder();

        if let Some(fields) = projection_fields {
            let expression = fields.join(", ");
            request_keys = request_keys.projection_expression(expression);
        }

        // TODO: do multiple batches if id count > 100
        for track_id in track_ids {
            request_keys = request_keys.keys(HashMap::from([
                (
                    pk_key_name.to_owned(),
                    AttributeValue::S(TrackInDB::get_pk(station_id)),
                ),
                (
                    sk_key_name.to_owned(),
                    AttributeValue::S(TrackInDB::get_sk(*track_id)),
                ),
            ]))
        }

        let resp = self
            .context
            .db_client
            .batch_get_item()
            .request_items(&self.context.db_table, request_keys.build()?)
            .send()
            .await?;

        Ok(serde_dynamo::from_items(
            resp.responses
                .expect("responses key must be present")
                .get(&self.context.db_table)
                .expect("response with table name must be present")
                .to_vec(),
        )?)
    }

    pub async fn list_plays_of_track(
        &self,
        station_id: StationId,
        track_id: TrackId,
        limit: i32,
        next_key: Option<Ulid>,
    ) -> Result<(Vec<TrackPlayInDB>, Option<String>)> {
        let partition_datetime = if let Some(next_key) = next_key {
            next_key.datetime().into()
        } else {
            Utc::now()
        };

        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .index_name("gsi1")
            .key_condition_expression("#gsi1pk = :gsi1pk AND begins_with(#sk, :sk_prefix)")
            .filter_expression("begins_with(#pk, :pk_prefix)")
            .expression_attribute_names("#gsi1pk", field!(gsi1pk @ TrackPlayInDB))
            .expression_attribute_names("#sk", field!(sk @ TrackPlayInDB))
            .expression_attribute_names("#pk", field!(pk @ TrackPlayInDB))
            .expression_attribute_values(
                ":gsi1pk",
                AttributeValue::S(TrackPlayInDB::get_gsi1pk(track_id, &partition_datetime)),
            )
            .expression_attribute_values(
                ":sk_prefix",
                AttributeValue::S(TrackPlayInDB::get_sk_prefix()),
            )
            // double check if it's the same station we're looking for
            .expression_attribute_values(
                ":pk_prefix",
                AttributeValue::S(PlayInDB::get_pk_station_prefix(station_id)),
            )
            .scan_index_forward(false)
            .limit(limit);

        if let Some(next_key) = next_key {
            // if random < (2 ** 80) - 1, assume no exclusive_start_key
            if next_key.random() < ULID_RANDOM_MAX {
                let play_id = next_key.into();
                query = query
                    .exclusive_start_key(
                        "pk",
                        AttributeValue::S(PlayInDB::get_pk(station_id, &partition_datetime)),
                    )
                    .exclusive_start_key(
                        "gsi1pk",
                        AttributeValue::S(TrackPlayInDB::get_gsi1pk(track_id, &partition_datetime)),
                    )
                    .exclusive_start_key("sk", AttributeValue::S(PlayInDB::get_sk(play_id)));
            }
        }

        let resp = query.send().await?;

        let next_key = if let Some(last_evaluated_key) = resp.last_evaluated_key {
            let paginate_key: Gsi1PaginateKey = serde_dynamo::from_item(last_evaluated_key)?;
            Some(
                paginate_key
                    .sk
                    .strip_prefix(&TrackPlayInDB::get_sk_prefix())
                    .expect("parse next key")
                    .to_owned(),
            )
        } else {
            let next_partition_datetime = truncate_datetime_to_months(partition_datetime)
                .expect("truncate datetime to months")
                - Duration::nanoseconds(1);

            let track_creation: DateTime<Utc> = track_id.datetime().into();
            if next_partition_datetime > track_creation {
                Some(
                    Ulid::from_parts(
                        next_partition_datetime
                            .timestamp_millis()
                            .try_into()
                            .expect("convert i64 unix ts to u64"),
                        u128::MAX,
                    )
                    .to_string(),
                )
            } else {
                None
            }
        };

        if let Some(items) = resp.items {
            Ok((serde_dynamo::from_items(items.to_vec())?, next_key))
        } else {
            Ok((vec![], next_key))
        }
    }
}
