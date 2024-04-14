use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use aws_sdk_dynamodb::types::{
    AttributeValue, KeysAndAttributes, Put, Select, TransactWriteItem, Update,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::Context;
use crate::{
    helpers::ziso_timestamp,
    models::{
        play::PlayInDB,
        station::{LatestPlay, StationInDB},
        track::{
            TrackInDB, TrackMetadataCreateInDB, TrackMetadataInDB, TrackMetadataKeys,
            TrackMinimalInDB,
        },
    },
};

pub trait Play {
    fn get_title(&self) -> &str;
    fn get_artist(&self) -> &str;
    fn is_song(&self) -> bool;
}

#[derive(Debug, Serialize)]
pub struct AddPlayResult {
    #[serde(flatten)]
    pub add_type: AddPlayType,
    pub play_id: Ulid,
    pub track_id: Ulid,
    metadata: AddPlayMetadata,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum AddPlayType {
    ExistingPlay,
    NewPlay,
    NewTrack,
}

impl From<AddPlayTypeInteral> for AddPlayType {
    fn from(value: AddPlayTypeInteral) -> Self {
        match value {
            AddPlayTypeInteral::ExistingPlay { .. } => AddPlayType::ExistingPlay,
            AddPlayTypeInteral::NewPlay { .. } => AddPlayType::NewPlay,
            AddPlayTypeInteral::NewTrack => AddPlayType::NewTrack,
        }
    }
}

#[derive(Debug)]
enum AddPlayTypeInteral {
    ExistingPlay { track_id: Ulid, play_id: Ulid },
    NewPlay { track_id: Ulid },
    NewTrack,
}

#[derive(Debug, Serialize)]
pub struct AddPlayMetadata {
    title: String,
    artist: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PaginateKey {
    pk: String,
    sk: String,
}

pub struct CRUDStation {
    context: Arc<Context>,
}

impl CRUDStation {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    pub async fn list(&self, limit: i32) -> Result<Vec<StationInDB>> {
        let resp = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk)")
            .expression_attribute_values(":pk", AttributeValue::S(StationInDB::get_pk()))
            .expression_attribute_values(":sk", AttributeValue::S(StationInDB::get_sk_prefix()))
            .select(Select::AllAttributes)
            .limit(limit)
            .send()
            .await?;

        let items = resp.items().to_vec();
        let stations: Vec<StationInDB> = serde_dynamo::from_items(items)?;

        Ok(stations)
    }

    pub async fn add_play(&self, station: &StationInDB, play: impl Play) -> Result<AddPlayResult> {
        let artist = play.get_artist();
        let title = play.get_title();

        let add_type = self.evaluate_play_metadata(station, artist, title).await?;
        let (result_track_id, result_play_id) = match &add_type {
            AddPlayTypeInteral::ExistingPlay { track_id, play_id } => {
                // all metadata matched, update play updated_ts only
                self.add_play_with_existing_play(station.id, *track_id, *play_id)
                    .await?;

                (*track_id, *play_id)
            }
            AddPlayTypeInteral::NewPlay { track_id } => {
                // insert new play with existing track
                let play = PlayInDB::new(station.id, *track_id);
                let play_id = play.id;

                // use the metadata from fetcher to populate latest_play
                self.add_play_with_new_play(station, play, artist, title)
                    .await?;

                (*track_id, play_id)
            }
            AddPlayTypeInteral::NewTrack => {
                // insert new track and play
                let track = TrackInDB::new(station.id, artist, title, play.is_song());
                let play = PlayInDB::new(station.id, track.id);

                let track_id = track.id;
                let play_id = play.id;

                self.add_play_with_new_track(station, track, play).await?;

                (track_id, play_id)
            }
        };

        Ok(AddPlayResult {
            add_type: AddPlayType::from(add_type),
            play_id: result_play_id,
            track_id: result_track_id,
            metadata: AddPlayMetadata {
                title: title.to_owned(),
                artist: artist.to_owned(),
            },
        })
    }

    async fn evaluate_play_metadata(
        &self,
        station: &StationInDB,
        artist: &str,
        title: &str,
    ) -> Result<AddPlayTypeInteral> {
        if let Some(latest_play) = &station.latest_play {
            if latest_play.artist == artist && latest_play.title == title {
                return Ok(AddPlayTypeInteral::ExistingPlay {
                    track_id: latest_play.track_id,
                    play_id: latest_play.id,
                });
            }
        }

        if let Some(track_metadata) = self.get_track_by_metadata(station, artist, title).await? {
            Ok(AddPlayTypeInteral::NewPlay {
                track_id: track_metadata.track_id,
            })
        } else {
            Ok(AddPlayTypeInteral::NewTrack)
        }
    }

    async fn add_play_with_existing_play(
        &self,
        station_id: Ulid,
        track_id: Ulid,
        play_id: Ulid,
    ) -> Result<()> {
        let play_datetime: DateTime<Utc> = play_id.datetime().into();

        self.context
            .db_client
            .update_item()
            .table_name(&self.context.db_table)
            .key(
                "pk",
                AttributeValue::S(PlayInDB::get_pk(station_id, &play_datetime)),
            )
            .key("sk", AttributeValue::S(PlayInDB::get_sk(play_id)))
            .condition_expression("id = :play_id AND track_id = :track_id")
            .update_expression("SET updated_ts = :ts")
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(":track_id", AttributeValue::S(track_id.to_string()))
            .expression_attribute_values(":ts", AttributeValue::S(ziso_timestamp(&Utc::now())))
            .send()
            .await?;

        Ok(())
    }

    async fn add_play_with_new_play(
        &self,
        station: &StationInDB,
        play: PlayInDB,
        artist: &str,
        title: &str,
    ) -> Result<()> {
        let play_id = play.id;
        let track_id = play.track_id;

        let latest_play = LatestPlay {
            id: play_id,
            track_id,
            artist: artist.to_owned(),
            title: title.to_owned(),
        };

        let play_put = Put::builder()
            .table_name(&self.context.db_table)
            .set_item(Some(serde_dynamo::to_item(play)?))
            .build()?;

        let track_update = Update::builder()
            .table_name(&self.context.db_table)
            .key("pk", AttributeValue::S(TrackInDB::get_pk(station.id)))
            .key("sk", AttributeValue::S(TrackInDB::get_sk(track_id)))
            .update_expression(
                "SET updated_ts = :ts, latest_play_id = :play_id, play_count = play_count + :inc",
            )
            .expression_attribute_values(":ts", AttributeValue::S(ziso_timestamp(&Utc::now())))
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
            .build()?;

        // update station with latest play
        let station_update = Update::builder()
            .table_name(&self.context.db_table)
            .key("pk", AttributeValue::S(StationInDB::get_pk()))
            .key("sk", AttributeValue::S(StationInDB::get_sk(station.id)))
            .update_expression(
                "SET updated_ts = :ts, latest_play = :latest_play, play_count = play_count + :inc",
            )
            .condition_expression("updated_ts = :station_locked_ts")
            .expression_attribute_values(":ts", AttributeValue::S(ziso_timestamp(&Utc::now())))
            .expression_attribute_values(
                ":latest_play",
                AttributeValue::M(serde_dynamo::to_item(latest_play)?),
            )
            .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
            .expression_attribute_values(
                ":station_locked_ts",
                AttributeValue::S(ziso_timestamp(&station.updated_ts)),
            )
            .build()?;

        // TODO handle errors
        let _resp = self
            .context
            .db_client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(play_put).build())
            .transact_items(TransactWriteItem::builder().update(track_update).build())
            .transact_items(TransactWriteItem::builder().update(station_update).build())
            .send()
            .await?;

        Ok(())
    }

    async fn add_play_with_new_track(
        &self,
        station: &StationInDB,
        mut track: TrackInDB,
        play: PlayInDB,
    ) -> Result<()> {
        let play_id = play.id;
        let track_id = track.id;

        track.latest_play_id = Some(play_id);
        track.play_count += 1;

        let track_metadata = TrackMetadataCreateInDB::from(&track);

        let latest_play = LatestPlay {
            id: play_id,
            track_id,
            artist: track.artist.clone(),
            title: track.title.clone(),
        };

        let track_put = Put::builder()
            .table_name(&self.context.db_table)
            .set_item(Some(serde_dynamo::to_item(track)?))
            .build()?;

        let track_metadata_put = Put::builder()
            .table_name(&self.context.db_table)
            .set_item(Some(serde_dynamo::to_item(track_metadata)?))
            .build()?;

        let play_put = Put::builder()
            .table_name(&self.context.db_table)
            .set_item(Some(serde_dynamo::to_item(play)?))
            .build()?;

        // update station with latest play and track
        let station_update_base = Update::builder()
            .table_name(&self.context.db_table)
            .key("pk", AttributeValue::S(StationInDB::get_pk()))
            .key("sk", AttributeValue::S(StationInDB::get_sk(station.id)))
            .expression_attribute_values(":ts", AttributeValue::S(ziso_timestamp(&Utc::now())))
            .expression_attribute_values(
                ":latest_play",
                AttributeValue::M(serde_dynamo::to_item(latest_play)?),
            )
            .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
            .expression_attribute_values(
                ":station_locked_ts",
                AttributeValue::S(ziso_timestamp(&station.updated_ts)),
            );

        let station_update = if station.first_play_id.is_none() {
            // update first play id as well if this is the first play
            station_update_base
                .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
                .update_expression(
                "SET updated_ts = :ts, first_play_id = :play_id, latest_play = :latest_play, play_count = play_count + :inc, track_count = track_count + :inc"
            )
            .condition_expression("updated_ts = :station_locked_ts AND first_play_id = :null")
            .expression_attribute_values(":null", AttributeValue::Null(true))
        } else {
            station_update_base.update_expression(
                "SET updated_ts = :ts, latest_play = :latest_play, play_count = play_count + :inc, track_count = track_count + :inc"
            )
            .condition_expression("updated_ts = :station_locked_ts")
        }
        .build()?;

        // TODO handle errors
        let _resp = self
            .context
            .db_client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(track_put).build())
            .transact_items(TransactWriteItem::builder().put(track_metadata_put).build())
            .transact_items(TransactWriteItem::builder().put(play_put).build())
            .transact_items(TransactWriteItem::builder().update(station_update).build())
            .send()
            .await?;

        Ok(())
    }

    pub async fn list_tracks(
        &self,
        station_id: Ulid,
        limit: i32,
        next_key: Option<Ulid>,
    ) -> Result<(Vec<TrackInDB>, Option<Ulid>)> {
        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk)")
            .expression_attribute_values(":pk", AttributeValue::S(TrackInDB::get_pk(station_id)))
            .expression_attribute_values(":sk", AttributeValue::S(TrackInDB::get_sk_prefix()))
            .scan_index_forward(false)
            .select(Select::AllAttributes)
            .limit(limit);

        if let Some(next_key) = next_key {
            query = query
                .exclusive_start_key("pk", AttributeValue::S(TrackInDB::get_pk(station_id)))
                .exclusive_start_key("sk", AttributeValue::S(TrackInDB::get_sk(next_key)));
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
        station_id: Ulid,
        artist: &str,
        limit: i32,
        next_key: Option<&str>,
    ) -> Result<(Vec<TrackInDB>, Option<String>)> {
        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(TrackMetadataInDB::get_pk(station_id, artist)),
            )
            .expression_attribute_values(
                ":sk",
                AttributeValue::S(TrackMetadataInDB::get_sk_prefix()),
            )
            .scan_index_forward(false)
            .select(Select::SpecificAttributes)
            .projection_expression("track_id")
            .limit(limit);

        if let Some(next_key) = next_key {
            query = query
                .exclusive_start_key(
                    "pk",
                    AttributeValue::S(TrackMetadataInDB::get_pk(station_id, artist)),
                )
                .exclusive_start_key("sk", AttributeValue::S(TrackMetadataInDB::get_sk(next_key)));
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

    pub async fn get_track(&self, station_id: Ulid, track_id: Ulid) -> Result<Option<TrackInDB>> {
        let resp = self
            .context
            .db_client
            .get_item()
            .table_name(&self.context.db_table)
            .key("pk", AttributeValue::S(TrackInDB::get_pk(station_id)))
            .key("sk", AttributeValue::S(TrackInDB::get_sk(track_id)))
            .send()
            .await?;

        if let Some(item) = resp.item {
            Ok(Some(serde_dynamo::from_item(item)?))
        } else {
            Ok(None)
        }
    }

    pub async fn batch_get_tracks(
        &self,
        station_id: Ulid,
        track_ids: impl Iterator<Item = &Ulid>,
    ) -> Result<Vec<TrackInDB>> {
        self.batch_get_tracks_internal(station_id, track_ids, None)
            .await
    }

    pub async fn batch_get_tracks_minimal(
        &self,
        station_id: Ulid,
        track_ids: impl Iterator<Item = &Ulid>,
    ) -> Result<Vec<TrackMinimalInDB>> {
        self.batch_get_tracks_internal(station_id, track_ids, Some("id, title, artist, is_song"))
            .await
    }

    async fn batch_get_tracks_internal<'a, O>(
        &self,
        station_id: Ulid,
        track_ids: impl Iterator<Item = &Ulid>,
        projection_expression: Option<&str>,
    ) -> Result<Vec<O>>
    where
        O: Serialize + Deserialize<'a>,
    {
        let mut request_keys = KeysAndAttributes::builder();

        if let Some(expression) = projection_expression {
            request_keys = request_keys.projection_expression(expression);
        }

        // TODO do multiple batches if id count > 100
        for track_id in track_ids {
            request_keys = request_keys.keys(HashMap::from([
                (
                    "pk".to_owned(),
                    AttributeValue::S(TrackInDB::get_pk(station_id)),
                ),
                (
                    "sk".to_owned(),
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
                "pk",
                AttributeValue::S(TrackMetadataInDB::get_pk(station.id, artist)),
            )
            .key("sk", AttributeValue::S(TrackMetadataInDB::get_sk(title)))
            .projection_expression("track_id")
            .consistent_read(true)
            .send()
            .await?;

        if let Some(item) = resp.item {
            Ok(Some(serde_dynamo::from_item(item)?))
        } else {
            Ok(None)
        }
    }

    // todo: traverse play partitions
    pub async fn list_plays(
        &self,
        station_id: Ulid,
        limit: i32,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        next_key: Option<Ulid>,
    ) -> Result<(Vec<PlayInDB>, Option<Ulid>)> {
        let play_datetime = if let Some(start) = start {
            start
        } else {
            Utc::now()
        };

        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND sk BETWEEN :start_sk AND :end_sk")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(PlayInDB::get_pk(station_id, &play_datetime)),
            )
            .expression_attribute_values(
                ":start_sk",
                AttributeValue::S(
                    PlayInDB::get_sk_prefix()
                        + &{
                            if let Some(start) = start {
                                Ulid::from_parts(start.timestamp_millis().try_into()?, 0)
                                    .to_string()
                            } else {
                                Ulid::nil().to_string()
                            }
                        },
                ),
            )
            .expression_attribute_values(
                ":end_sk",
                AttributeValue::S(
                    PlayInDB::get_sk_prefix()
                        + &{
                            if let Some(end) = end {
                                Ulid::from_parts(end.timestamp_millis().try_into()?, u128::MAX)
                                    .to_string()
                            } else {
                                Ulid::from_parts(u64::MAX, u128::MAX).to_string()
                            }
                        },
                ),
            )
            .select(Select::AllAttributes)
            .limit(limit);

        if let Some(next_key) = next_key {
            let next_key_datetime: DateTime<Utc> = next_key.datetime().into();

            query = query
                .exclusive_start_key(
                    "pk",
                    AttributeValue::S(PlayInDB::get_pk(station_id, &next_key_datetime)),
                )
                .exclusive_start_key("sk", AttributeValue::S(PlayInDB::get_sk(next_key)));
        }

        let resp = query.send().await?;

        let next_key = if let Some(last_evaluated_key) = resp.last_evaluated_key {
            let paginate_key: PaginateKey = serde_dynamo::from_item(last_evaluated_key)?;
            Some(
                Ulid::from_string(
                    paginate_key
                        .sk
                        .strip_prefix(&PlayInDB::get_sk_prefix())
                        .expect("parse next key"),
                )
                .expect("next key into ulid"),
            )
        } else {
            None
        };

        if let Some(items) = resp.items {
            Ok((serde_dynamo::from_items(items.to_vec())?, next_key))
        } else {
            Ok((vec![], next_key))
        }
    }
}
