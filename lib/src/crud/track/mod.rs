pub mod models;
mod provider;

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::crud::play::models::PlayInDB;
use crate::crud::shared::models::{Gsi1PaginateKey, PaginateKey};
use crate::crud::station::models::{StationId, StationInDB};
use crate::crud::track::models::TrackId;
use crate::crud::track::models::{
    TrackInDB, TrackMetadataInDB, TrackMetadataKeys, TrackMinimalInDB, TrackPlayInDB,
};
use crate::crud::Context;
use crate::helpers::truncate_datetime_to_months;
use provider::{
    BatchGetItemInput, DynamoDBProvider, ExclusiveStartKey, GetItemConfig, GetItemInput,
    ProjectedFields, QueryPrefixConfig, QueryPrefixInput,
};

use self::provider::{
    BatchGetItemConfig, BatchGetItemKey, Gsi1ExclusiveStartKey, QueryPrefixGsi1Config,
    QueryPrefixGsi1Input,
};

const ULID_RANDOM_MAX: u128 = (1 << 80) - 1;

pub struct CRUDTrack {
    provider: DynamoDBProvider,
}

impl CRUDTrack {
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            provider: DynamoDBProvider::new(context),
        }
    }

    pub async fn get_track(
        &self,
        station_id: StationId,
        track_id: TrackId,
    ) -> Result<Option<TrackInDB>> {
        let resp = self
            .provider
            .get_item(
                GetItemInput {
                    pk: TrackInDB::get_pk(station_id),
                    sk: TrackInDB::get_sk(track_id),
                },
                GetItemConfig {
                    consistent_read: false,
                    projected_fields: ProjectedFields::All,
                },
            )
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
            .provider
            .get_item(
                GetItemInput {
                    pk: TrackMetadataInDB::get_pk(station.id, artist),
                    sk: TrackMetadataInDB::get_sk(title),
                },
                GetItemConfig {
                    consistent_read: true,
                    projected_fields: ProjectedFields::Some(&["track_id"]),
                },
            )
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
        let exclusive_start_key = if let Some(next_key) = next_key {
            let track_id = next_key.into();
            Some(ExclusiveStartKey {
                pk: TrackInDB::get_pk(station_id),
                sk: TrackInDB::get_sk(track_id),
            })
        } else {
            None
        };

        let resp = self
            .provider
            .query_prefix(
                QueryPrefixInput {
                    pk: TrackInDB::get_pk(station_id),
                    sk_prefix: TrackInDB::get_sk_prefix(),
                    scan_forward: false,
                    exclusive_start_key,
                },
                QueryPrefixConfig {
                    limit,
                    projected_fields: ProjectedFields::All,
                },
            )
            .await?;

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
        let exclusive_start_key = next_key.map(|next_key| ExclusiveStartKey {
            pk: TrackMetadataInDB::get_pk(station_id, artist),
            sk: TrackMetadataInDB::get_sk(next_key),
        });

        let resp = self
            .provider
            .query_prefix(
                QueryPrefixInput {
                    pk: TrackMetadataInDB::get_pk(station_id, artist),
                    sk_prefix: TrackMetadataInDB::get_sk_prefix(),
                    scan_forward: false,
                    exclusive_start_key,
                },
                QueryPrefixConfig {
                    limit,
                    projected_fields: ProjectedFields::Some(&["track_id"]),
                },
            )
            .await?;

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
        self.batch_get_tracks_internal(station_id, track_ids, ProjectedFields::All)
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
            ProjectedFields::Some(&["id", "title", "artist", "is_song"]),
        )
        .await
    }

    async fn batch_get_tracks_internal<'a, O>(
        &self,
        station_id: StationId,
        track_ids: impl Iterator<Item = &TrackId>,
        projected_fields: ProjectedFields,
    ) -> Result<Vec<O>>
    where
        O: Serialize + Deserialize<'a>,
    {
        let resp = self
            .provider
            .batch_get_item(
                BatchGetItemInput {
                    keys: track_ids.map(|id| BatchGetItemKey {
                        pk: TrackInDB::get_pk(station_id),
                        sk: TrackInDB::get_sk(*id),
                    }),
                },
                BatchGetItemConfig { projected_fields },
            )
            .await?;

        Ok(serde_dynamo::from_items(
            resp.responses
                .expect("responses key must be present")
                .get(self.provider.table_name())
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

        let exclusive_start_key = if let Some(next_key) = next_key {
            // if random < (2 ** 80) - 1, assume no exclusive_start_key
            if next_key.random() < ULID_RANDOM_MAX {
                let play_id = next_key.into();
                Some(Gsi1ExclusiveStartKey {
                    gsi1pk: TrackPlayInDB::get_gsi1pk(track_id, &partition_datetime),
                    sk: PlayInDB::get_sk(play_id),
                    pk: PlayInDB::get_pk(station_id, &partition_datetime),
                })
            } else {
                None
            }
        } else {
            None
        };

        let resp = self
            .provider
            .query_prefix_gsi1(
                QueryPrefixGsi1Input {
                    gsi1pk: TrackPlayInDB::get_gsi1pk(track_id, &partition_datetime),
                    sk_prefix: TrackPlayInDB::get_sk_prefix(),
                    // double check if it's the same station we're looking for
                    pk_prefix: Some(PlayInDB::get_pk_station_prefix(station_id)),
                    scan_forward: false,
                    exclusive_start_key,
                },
                QueryPrefixGsi1Config { limit },
            )
            .await?;

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
