use std::collections::HashMap;

use anyhow::{bail, Result};
use aws_sdk_dynamodb::{
    types::{AttributeValue, KeysAndAttributes, Put, Select, TransactWriteItem, Update},
    Client,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    helpers::ziso_timestamp,
    models::{
        play::PlayInDB,
        station::StationInDB,
        track::{TrackInDB, TrackMinimalInDB},
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
    db_client: Client,
    db_table: String,
}

impl CRUDStation {
    pub fn new(client: Client, table_name: &str) -> Self {
        Self {
            db_client: client,
            db_table: table_name.to_owned(),
        }
    }

    pub async fn list(&self, limit: i32) -> Result<Vec<StationInDB>> {
        let resp = self
            .db_client
            .query()
            .table_name(&self.db_table)
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
        let title = play.get_title();
        let artist = play.get_artist();

        // get the fetched play's track
        let maybe_track = self.get_track_by_name(station, artist, title).await?;

        let result_play_id;
        let result_track_id;

        let add_type = if let (Some(station_latest_play_track_id), Some(track_id)) = (
            station.latest_play_track_id,
            maybe_track.as_ref().map(|track| track.id),
        ) {
            result_track_id = station_latest_play_track_id;

            if station.latest_play_id.is_some() && station_latest_play_track_id == track_id {
                // update play updated_ts only
                let station_latest_play_id =
                    station.latest_play_id.expect("station has latest play id");

                result_play_id = station_latest_play_id;

                self.add_play_with_existing_play(
                    station.id,
                    station_latest_play_track_id,
                    station_latest_play_id,
                )
                .await?
            } else {
                // insert new play using that track id
                let play = PlayInDB::new(station.id, track_id);

                result_play_id = play.id;

                self.add_play_with_new_play(station, play.clone()).await?
            }
        } else {
            // insert new track and play
            let track = TrackInDB::new(station.id, artist, title, play.is_song(), None);
            let play = PlayInDB::new(station.id, track.id);

            result_track_id = track.id;
            result_play_id = play.id;

            self.add_play_with_new_track(station, track, play.clone())
                .await?
        };

        Ok(AddPlayResult {
            add_type,
            play_id: result_play_id,
            track_id: result_track_id,
            metadata: AddPlayMetadata {
                title: title.to_owned(),
                artist: artist.to_owned(),
            },
        })
    }

    async fn add_play_with_existing_play(
        &self,
        station_id: Ulid,
        track_id: Ulid,
        play_id: Ulid,
    ) -> Result<AddPlayType> {
        let play_datetime: DateTime<Utc> = play_id.datetime().into();
        let play_partition = play_datetime.format("%Y-%m-%d");

        self.db_client
            .update_item()
            .table_name(&self.db_table)
            .key(
                "pk",
                AttributeValue::S(PlayInDB::get_pk(station_id, &play_partition.to_string())),
            )
            .key("sk", AttributeValue::S(PlayInDB::get_sk(play_id)))
            .condition_expression("id = :play_id AND track_id = :track_id")
            .update_expression("SET updated_ts = :ts")
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(":track_id", AttributeValue::S(track_id.to_string()))
            .expression_attribute_values(":ts", AttributeValue::S(ziso_timestamp(&Utc::now())))
            .send()
            .await?;

        Ok(AddPlayType::ExistingPlay)
    }
    async fn add_play_with_new_play(
        &self,
        station: &StationInDB,
        play: PlayInDB,
    ) -> Result<AddPlayType> {
        let play_id = play.id;
        let track_id = play.track_id;

        let play_put = Put::builder()
            .table_name(&self.db_table)
            .set_item(Some(serde_dynamo::to_item(play)?))
            .build()?;

        let track_update = Update::builder()
            .table_name(&self.db_table)
            .key("pk", AttributeValue::S(TrackInDB::get_pk(station.id)))
            .key("sk", AttributeValue::S(TrackInDB::get_sk(track_id)))
            .update_expression("SET updated_ts = :ts, latest_play_id = :play_id")
            .expression_attribute_values(":ts", AttributeValue::S(ziso_timestamp(&Utc::now())))
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .build()?;

        // update station with latest play
        let station_update = Update::builder()
            .table_name(&self.db_table)
            .key("pk", AttributeValue::S(StationInDB::get_pk()))
            .key("sk", AttributeValue::S(StationInDB::get_sk(station.id)))
            .update_expression(
                "SET updated_ts = :ts, latest_play_id = :play_id, latest_play_track_id = :track_id, play_count = play_count + :inc",
            )
            .condition_expression("updated_ts = :station_locked_ts")
            .expression_attribute_values(
                ":ts",
                AttributeValue::S(ziso_timestamp(&Utc::now()))
            )
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(":track_id", AttributeValue::S(track_id.to_string()))
            .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
            .expression_attribute_values(
                ":station_locked_ts",
                AttributeValue::S(
                    ziso_timestamp(&station.updated_ts)
                ),
            )
            .build()?;

        // TODO handle errors
        let _resp = self
            .db_client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(play_put).build())
            .transact_items(TransactWriteItem::builder().update(track_update).build())
            .transact_items(TransactWriteItem::builder().update(station_update).build())
            .send()
            .await?;

        Ok(AddPlayType::NewPlay)
    }
    async fn add_play_with_new_track(
        &self,
        station: &StationInDB,
        mut track: TrackInDB,
        play: PlayInDB,
    ) -> Result<AddPlayType> {
        let play_id = play.id;
        let track_id = track.id;

        track.latest_play_id = Some(play_id);

        let track_put = Put::builder()
            .table_name(&self.db_table)
            .set_item(Some(serde_dynamo::to_item(track)?))
            .build()?;

        let play_put = Put::builder()
            .table_name(&self.db_table)
            .set_item(Some(serde_dynamo::to_item(play)?))
            .build()?;

        // update station with latest play and track
        let station_update_base = Update::builder()
            .table_name(&self.db_table)
            .key("pk", AttributeValue::S(StationInDB::get_pk()))
            .key("sk", AttributeValue::S(StationInDB::get_sk(station.id)))
            .expression_attribute_values(":ts", AttributeValue::S(ziso_timestamp(&Utc::now())))
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(":track_id", AttributeValue::S(track_id.to_string()))
            .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
            .expression_attribute_values(
                ":station_locked_ts",
                AttributeValue::S(ziso_timestamp(&station.updated_ts)),
            );

        let station_update = if station.first_play_id.is_none() {
            // update first play id as well if this is the first play
            station_update_base.update_expression(
                "SET updated_ts = :ts, first_play_id = :play_id, latest_play_id = :play_id, latest_play_track_id = :track_id, play_count = play_count + :inc, track_count = track_count + :inc"
            )
            .condition_expression("updated_ts = :station_locked_ts AND first_play_id = :null")
            .expression_attribute_values(":null", AttributeValue::Null(true))
        } else {
            station_update_base.update_expression(
                "SET updated_ts = :ts, latest_play_id = :play_id, latest_play_track_id = :track_id, play_count = play_count + :inc, track_count = track_count + :inc"
            )
            .condition_expression("updated_ts = :station_locked_ts")
        }
        .build()?;

        // TODO handle errors
        let _resp = self
            .db_client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(track_put).build())
            .transact_items(TransactWriteItem::builder().put(play_put).build())
            .transact_items(TransactWriteItem::builder().update(station_update).build())
            .send()
            .await?;

        Ok(AddPlayType::NewTrack)
    }

    pub async fn list_tracks(
        &self,
        station_id: Ulid,
        limit: i32,
        next_key: Option<Ulid>,
    ) -> Result<(Vec<TrackInDB>, Option<Ulid>)> {
        let mut query = self
            .db_client
            .query()
            .table_name(&self.db_table)
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

    pub async fn get_track(&self, station_id: Ulid, track_id: Ulid) -> Result<Option<TrackInDB>> {
        let resp = self
            .db_client
            .get_item()
            .table_name(&self.db_table)
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
    ) -> Result<Vec<TrackMinimalInDB>> {
        let mut request_keys =
            KeysAndAttributes::builder().projection_expression("id, title, artist, is_song");

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
            .db_client
            .batch_get_item()
            .request_items(&self.db_table, request_keys.build()?)
            .send()
            .await?;

        Ok(serde_dynamo::from_items(
            resp.responses
                .expect("responses key must be present")
                .get(&self.db_table)
                .expect("response with table name must be present")
                .to_vec(),
        )?)
    }

    pub async fn get_track_by_name(
        &self,
        station: &StationInDB,
        artist: &str,
        title: &str,
    ) -> Result<Option<TrackInDB>> {
        let resp = self
            .db_client
            .query()
            .table_name(&self.db_table)
            .index_name("gsi1")
            .key_condition_expression("gsi1pk = :gsi1pk AND gsi1sk = :gsi1sk")
            .expression_attribute_values(
                ":gsi1pk",
                AttributeValue::S(TrackInDB::get_gsi1pk(station.id, artist)),
            )
            .expression_attribute_values(":gsi1sk", AttributeValue::S(TrackInDB::get_gsi1sk(title)))
            .select(Select::AllAttributes)
            .limit(1)
            .send()
            .await?;

        match resp.count {
            0 => Ok(None),
            1 => Ok(if let Some(item) = resp.items().iter().nth(0).cloned() {
                serde_dynamo::from_item(item)?
            } else {
                bail!("kaputt state!")
            }),
            _ => bail!("unexpected multiple items"),
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
        let play_partition = {
            let play_datetime = if let Some(start) = start {
                start
            } else {
                Utc::now()
            };

            play_datetime.format("%Y-%m-%d")
        };

        let mut query = self
            .db_client
            .query()
            .table_name(&self.db_table)
            .key_condition_expression("pk = :pk AND sk BETWEEN :start_sk AND :end_sk")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(PlayInDB::get_pk(station_id, &play_partition.to_string())),
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
            .scan_index_forward(false)
            .select(Select::AllAttributes)
            .limit(limit);

        if let Some(next_key) = next_key {
            let next_key_datetime: DateTime<Utc> = next_key.datetime().into();
            let next_key_partition = next_key_datetime.format("%Y-%m-%d");

            query = query
                .exclusive_start_key(
                    "pk",
                    AttributeValue::S(PlayInDB::get_pk(
                        station_id,
                        &next_key_partition.to_string(),
                    )),
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
