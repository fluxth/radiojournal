use std::collections::HashMap;

use anyhow::{bail, Result};
use aws_sdk_dynamodb::{
    types::{AttributeValue, KeysAndAttributes, Put, Select, TransactWriteItem, Update},
    Client,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use ulid::Ulid;

use crate::models::{PlayInDB, StationInDB, TrackInDB, TrackMinimalInDB};

pub trait Play {
    fn get_title(&self) -> &str;
    fn get_artist(&self) -> &str;
    fn is_song(&self) -> bool;
}

#[derive(Debug, Serialize)]
pub struct AddPlayResult {
    #[serde(flatten)]
    add_type: AddPlayType,
    play_id: Ulid,
    track_id: Ulid,
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

#[derive(Clone)]
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

    pub async fn list(&self) -> Result<Vec<StationInDB>> {
        let resp = self
            .db_client
            .query()
            .table_name(&self.db_table)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk)")
            .expression_attribute_values(":pk", AttributeValue::S("STATIONS".to_owned()))
            .expression_attribute_values(":sk", AttributeValue::S("STATION#".to_owned()))
            .select(Select::AllAttributes)
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

                self.add_play_with_existing_play(station.id, station_latest_play_id)
                    .await?
            } else {
                // insert new play using that track id
                let play = PlayInDB::new(station.id, station_latest_play_track_id);

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
        play_id: Ulid,
    ) -> Result<AddPlayType> {
        let play_datetime: DateTime<Utc> = play_id.datetime().try_into()?;
        let play_partition = play_datetime.format("%Y-%m-%d");

        self.db_client
            .update_item()
            .table_name(&self.db_table)
            .key(
                "pk",
                AttributeValue::S(format!("STATION#{}#PLAYS#{}", station_id, play_partition)),
            )
            .key("sk", AttributeValue::S(format!("PLAY#{}", play_id)))
            .condition_expression("id = :play_id")
            .update_expression("SET updated_ts = :ts")
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(
                ":ts",
                AttributeValue::S(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)),
            )
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
            .key(
                "pk",
                AttributeValue::S(format!("STATION#{}#TRACKS", station.id)),
            )
            .key("sk", AttributeValue::S(format!("TRACK#{}", track_id)))
            .update_expression("SET updated_ts = :ts, latest_play_id = :play_id")
            .expression_attribute_values(
                ":ts",
                AttributeValue::S(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)),
            )
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .build()?;

        // update station with latest play
        let station_update = Update::builder()
            .table_name(&self.db_table)
            .key("pk", AttributeValue::S("STATIONS".to_owned()))
            .key("sk", AttributeValue::S(format!("STATION#{}", station.id)))
            .update_expression(
                "SET updated_ts = :ts, latest_play_id = :play_id, latest_play_track_id = :track_id, play_count = play_count + :inc",
            )
            .condition_expression("updated_ts = :station_locked_ts")
            .expression_attribute_values(
                ":ts",
                AttributeValue::S(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)),
            )
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(":track_id", AttributeValue::S(track_id.to_string()))
            .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
            .expression_attribute_values(
                ":station_locked_ts",
                AttributeValue::S(
                    station
                        .updated_ts
                        .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                ),
            )
            .build()?;

        let resp = self
            .db_client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(play_put).build())
            .transact_items(TransactWriteItem::builder().update(track_update).build())
            .transact_items(TransactWriteItem::builder().update(station_update).build())
            .send()
            .await?;

        dbg!(resp);

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
        let station_update = Update::builder()
            .table_name(&self.db_table)
            .key("pk", AttributeValue::S("STATIONS".to_owned()))
            .key("sk", AttributeValue::S(format!("STATION#{}", station.id)))
            .update_expression(
                "SET updated_ts = :ts, latest_play_id = :play_id, latest_play_track_id = :track_id, play_count = play_count + :inc, track_count = track_count + :inc",
            )
            .condition_expression("updated_ts = :station_locked_ts")
            .expression_attribute_values(
                ":ts",
                AttributeValue::S(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)),
            )
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(":track_id", AttributeValue::S(track_id.to_string()))
            .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
            .expression_attribute_values(
                ":station_locked_ts",
                AttributeValue::S(
                    station
                        .updated_ts
                        .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                ),
            )
            .build()?;

        let resp = self
            .db_client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(track_put).build())
            .transact_items(TransactWriteItem::builder().put(play_put).build())
            .transact_items(TransactWriteItem::builder().update(station_update).build())
            .send()
            .await?;

        dbg!(resp);

        Ok(AddPlayType::NewTrack)
    }

    pub async fn get_track(&self, station_id: Ulid, track_id: Ulid) -> Result<Option<TrackInDB>> {
        let resp = self
            .db_client
            .get_item()
            .table_name(&self.db_table)
            .key(
                "pk",
                AttributeValue::S(format!("STATION#{}#TRACKS", station_id.to_string())),
            )
            .key(
                "sk",
                AttributeValue::S(format!("TRACK#{}", track_id.to_string())),
            )
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

        for track_id in track_ids {
            request_keys = request_keys.keys(HashMap::from([
                (
                    "pk".to_owned(),
                    AttributeValue::S(format!("STATION#{}#TRACKS", station_id)),
                ),
                (
                    "sk".to_owned(),
                    AttributeValue::S(format!("TRACK#{}", track_id)),
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
                AttributeValue::S(format!("STATION#{}#ARTIST#{}", station.id, artist)),
            )
            .expression_attribute_values(":gsi1sk", AttributeValue::S(format!("TITLE#{}", title)))
            .select(Select::AllAttributes)
            .send()
            .await?;

        match resp.count {
            0 => Ok(None),
            1 => Ok(
                if let Some(item) = resp.items().to_vec().into_iter().nth(0) {
                    serde_dynamo::from_item(item)?
                } else {
                    bail!("kaputt state!")
                },
            ),
            _ => bail!("unexpected multiple items"),
        }
    }

    // todo: traverse play partitions
    pub async fn list_plays(&self, station_id: Ulid) -> Result<Vec<PlayInDB>> {
        let play_datetime = Utc::now();
        let play_partition = play_datetime.format("%Y-%m-%d");

        let resp = self
            .db_client
            .query()
            .table_name(&self.db_table)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("STATION#{}#PLAYS#{}", station_id, play_partition)),
            )
            .expression_attribute_values(":sk", AttributeValue::S("PLAY#".to_owned()))
            .scan_index_forward(false)
            .select(Select::AllAttributes)
            .send()
            .await?;

        if let Some(items) = resp.items {
            Ok(serde_dynamo::from_items(items.to_vec())?)
        } else {
            Ok(vec![])
        }
    }
}
