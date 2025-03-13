pub mod models;
mod provider;

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::crud::Context;
use crate::crud::play::models::{PlayId, PlayInDB};
use crate::crud::station::models::{LatestPlay, StationId, StationInDB};
use crate::crud::track::CRUDTrack;
use crate::crud::track::models::{TrackId, TrackInDB, TrackMetadataCreateInDB};
use crate::helpers::ziso_timestamp;
use models::{AddPlayMetadata, AddPlayResult, AddPlayTypeInternal, Play};
use provider::{
    BuildStationUpdateInput, BuildTrackUpdateInput, DynamoDBProvider, StationUpdateIncrementType,
    TransactWriteItem, UpdatePlayInput, build_put, build_station_update, build_track_update,
};

pub struct CRUDLogger {
    provider: DynamoDBProvider,
    crud_track: CRUDTrack,
}

impl CRUDLogger {
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            crud_track: CRUDTrack::new(context.clone()),
            provider: DynamoDBProvider::new(context.clone()),
        }
    }

    pub async fn add_play(
        &self,
        station: &mut StationInDB,
        play: impl Play,
    ) -> Result<AddPlayResult> {
        let artist = play.get_artist();
        let title = play.get_title();

        let add_type = self.evaluate_play_metadata(station, artist, title).await?;
        let (result_track_id, result_play_id) = match &add_type {
            AddPlayTypeInternal::ExistingPlay { track_id, play_id } => {
                // all metadata matched, update play updated_ts only
                self.add_play_with_existing_play(station.id, *track_id, *play_id)
                    .await?;

                (*track_id, *play_id)
            }
            AddPlayTypeInternal::NewPlay { track_id } => {
                // insert new play with existing track
                let play = PlayInDB::new(station.id, *track_id);
                let play_id = play.id;

                // use the metadata from fetcher to populate latest_play
                self.add_play_with_new_play(station, play, artist, title)
                    .await?;

                (*track_id, play_id)
            }
            AddPlayTypeInternal::NewTrack => {
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
            add_type: add_type.into(),
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
        station: &mut StationInDB,
        artist: &str,
        title: &str,
    ) -> Result<AddPlayTypeInternal> {
        if let Some(latest_play) = &station.latest_play {
            if latest_play.artist == artist && latest_play.title == title {
                return Ok(AddPlayTypeInternal::ExistingPlay {
                    track_id: latest_play.track_id,
                    play_id: latest_play.id,
                });
            }
        }

        if let Some(track_metadata) = self
            .crud_track
            .get_track_by_metadata(station, artist, title)
            .await?
        {
            Ok(AddPlayTypeInternal::NewPlay {
                track_id: track_metadata.track_id,
            })
        } else {
            Ok(AddPlayTypeInternal::NewTrack)
        }
    }

    async fn add_play_with_existing_play(
        &self,
        station_id: StationId,
        track_id: TrackId,
        play_id: PlayId,
    ) -> Result<()> {
        let play_datetime: DateTime<Utc> = play_id.datetime().into();

        self.provider
            .update_play(UpdatePlayInput {
                pk: PlayInDB::get_pk(station_id, &play_datetime),
                sk: PlayInDB::get_sk(play_id),
                play_id: play_id.to_string(),
                track_id: track_id.to_string(),
                update_timestamp: ziso_timestamp(&Utc::now()),
            })
            .await?;

        Ok(())
    }

    async fn add_play_with_new_play(
        &self,
        station: &mut StationInDB,
        play: PlayInDB,
        artist: &str,
        title: &str,
    ) -> Result<()> {
        let latest_play = LatestPlay {
            id: play.id,
            track_id: play.track_id,
            artist: artist.to_owned(),
            title: title.to_owned(),
        };

        let now = Utc::now();

        let PreparedTransaction { items, callback } = build_new_play_transaction(
            self.provider.table_name(),
            station,
            &play,
            latest_play,
            now,
        )?;

        // TODO handle errors
        self.provider.transact_write_items(items).await?;

        callback(station, None);

        Ok(())
    }

    async fn add_play_with_new_track(
        &self,
        station: &mut StationInDB,
        mut track: TrackInDB,
        play: PlayInDB,
    ) -> Result<()> {
        track.latest_play_id = Some(play.id);
        track.play_count += 1;

        let track_metadata = TrackMetadataCreateInDB::from(&track);

        let latest_play = LatestPlay {
            id: play.id,
            track_id: track.id,
            artist: track.artist.clone(),
            title: track.title.clone(),
        };

        let now = Utc::now();

        let PreparedTransaction { items, callback } = build_new_track_and_play_transaction(
            self.provider.table_name(),
            station,
            &track,
            &track_metadata,
            &play,
            latest_play,
            now,
        )?;

        // TODO handle errors
        self.provider.transact_write_items(items).await?;

        callback(station);

        Ok(())
    }
}

#[derive(Error, Debug)]
enum BuildTransactionError {
    #[error("Failed to build transaction item: {0:?}")]
    BuildError(#[from] aws_sdk_dynamodb::error::BuildError),
    #[error("Failed to serialize into transaction item: {0:?}")]
    SerializeError(#[from] serde_dynamo::Error),
}

struct PreparedTransaction<CallbackFn> {
    items: Vec<TransactWriteItem>,
    callback: CallbackFn,
}

fn build_new_play_transaction<'i>(
    table_name: &'i str,
    station: &'i StationInDB,
    play: &'i PlayInDB,
    latest_play: LatestPlay,
    timestamp: DateTime<Utc>,
) -> Result<
    PreparedTransaction<impl FnOnce(&mut StationInDB, Option<&mut TrackInDB>)>,
    BuildTransactionError,
> {
    let play_put = build_put(table_name, serde_dynamo::to_item(play)?)?;

    let track_update = build_track_update(
        table_name,
        BuildTrackUpdateInput {
            pk: TrackInDB::get_pk(station.id),
            sk: TrackInDB::get_sk(play.track_id),
            play_id: play.id.to_string(),
            update_timestamp: ziso_timestamp(&timestamp),
        },
    )?;

    // update station with latest play
    let station_update = build_station_update(
        table_name,
        BuildStationUpdateInput {
            pk: StationInDB::get_pk(),
            sk: StationInDB::get_sk(station.id),
            increment: StationUpdateIncrementType::Play,
            latest_play: serde_dynamo::to_item(latest_play.clone())?,
            first_play_id: None, // first play will always create new track
            update_timestamp: ziso_timestamp(&timestamp),
            locked_timestamp: Some(ziso_timestamp(&station.updated_ts)),
        },
    )?;

    let play_id = play.id;
    let update_structs_callback =
        move |station: &mut StationInDB, track: Option<&mut TrackInDB>| {
            station.updated_ts = timestamp;
            station.latest_play = Some(latest_play);
            station.play_count += 1;

            if let Some(track) = track {
                track.updated_ts = timestamp;
                track.latest_play_id = Some(play_id);
                track.play_count += 1;
            }
        };

    Ok(PreparedTransaction {
        items: vec![
            TransactWriteItem::Put(play_put),
            TransactWriteItem::Update(track_update),
            TransactWriteItem::Update(station_update),
        ],
        callback: update_structs_callback,
    })
}

fn build_new_track_and_play_transaction<'i>(
    table_name: &'i str,
    station: &'i StationInDB,
    track: &'i TrackInDB,
    track_metadata: &'i TrackMetadataCreateInDB,
    play: &'i PlayInDB,
    latest_play: LatestPlay,
    timestamp: DateTime<Utc>,
) -> Result<PreparedTransaction<impl FnOnce(&mut StationInDB)>, BuildTransactionError> {
    let track_put = build_put(table_name, serde_dynamo::to_item(track)?)?;
    let track_metadata_put = build_put(table_name, serde_dynamo::to_item(track_metadata)?)?;
    let play_put = build_put(table_name, serde_dynamo::to_item(play)?)?;

    // update station with latest play and track
    let station_update = build_station_update(
        table_name,
        BuildStationUpdateInput {
            pk: StationInDB::get_pk(),
            sk: StationInDB::get_sk(station.id),
            increment: StationUpdateIncrementType::PlayAndTrack,
            latest_play: serde_dynamo::to_item(latest_play.clone())?,
            first_play_id: match station.first_play_id {
                None => Some(play.id.to_string()),
                Some(_) => None,
            },
            update_timestamp: ziso_timestamp(&timestamp),
            locked_timestamp: Some(ziso_timestamp(&station.updated_ts)),
        },
    )?;

    let play_id = play.id;
    let update_structs_callback = move |station: &mut StationInDB| {
        station.updated_ts = timestamp;
        station.latest_play = Some(latest_play);
        station.play_count += 1;
        station.track_count += 1;
        if station.first_play_id.is_none() {
            station.first_play_id = Some(play_id)
        }
    };

    Ok(PreparedTransaction {
        items: vec![
            TransactWriteItem::Put(track_put),
            TransactWriteItem::Put(track_metadata_put),
            TransactWriteItem::Put(play_put),
            TransactWriteItem::Update(station_update),
        ],
        callback: update_structs_callback,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use aws_sdk_dynamodb::types::AttributeValue;
    use ulid::Ulid;

    #[test]
    fn test_build_new_play_transaction() {
        let mut station = StationInDB::new_for_test();
        let track_id = Ulid::from_parts(1, 1).into();

        let new_play = PlayInDB::new(station.id, track_id);

        let latest_play = LatestPlay {
            id: Ulid::from_parts(2, 99).into(),
            track_id: Ulid::from_parts(1, 99).into(),
            artist: "artist".to_owned(),
            title: "title".to_owned(),
        };

        let timestamp = DateTime::from_timestamp(1, 0).unwrap();

        let PreparedTransaction { items, callback } = build_new_play_transaction(
            "tablename",
            &mut station,
            &new_play,
            latest_play.clone(),
            timestamp,
        )
        .unwrap();

        assert_eq!(items.len(), 3);

        match &items[0] {
            TransactWriteItem::Put(play_put) => {
                assert_eq!(play_put.table_name(), "tablename");
                let expected_item = serde_dynamo::to_item(new_play.clone()).unwrap();
                assert_eq!(play_put.item(), &expected_item);
            }
            _ => unreachable!(),
        };

        match &items[1] {
            TransactWriteItem::Update(track_update) => {
                assert_eq!(track_update.table_name(), "tablename");
                assert_eq!(
                    track_update.key(),
                    &HashMap::from_iter([
                        (
                            "pk".to_owned(),
                            AttributeValue::S(TrackInDB::get_pk(station.id))
                        ),
                        (
                            "sk".to_owned(),
                            AttributeValue::S(TrackInDB::get_sk(track_id))
                        )
                    ])
                );

                let values = track_update.expression_attribute_values().unwrap();
                assert_eq!(
                    values.get(":play_id").unwrap().as_s().unwrap(),
                    new_play.id.to_string().as_str()
                );
                assert_eq!(
                    values.get(":ts").unwrap().as_s().unwrap(),
                    &ziso_timestamp(&timestamp)
                );
            }
            _ => unreachable!(),
        }

        match &items[2] {
            TransactWriteItem::Update(station_update) => {
                assert_eq!(station_update.table_name(), "tablename");
                assert_eq!(
                    station_update.key(),
                    &HashMap::from_iter([
                        ("pk".to_owned(), AttributeValue::S(StationInDB::get_pk())),
                        (
                            "sk".to_owned(),
                            AttributeValue::S(StationInDB::get_sk(station.id))
                        )
                    ])
                );

                assert!(!station_update.update_expression().contains("track_count"));

                let values = station_update.expression_attribute_values().unwrap();

                let expected_item = serde_dynamo::to_item(latest_play.clone()).unwrap();
                assert_eq!(
                    values.get(":latest_play").unwrap().as_m().unwrap(),
                    &expected_item
                );

                assert!(!values.contains_key(":first_play_id"));
                assert_eq!(
                    values.get(":ts").unwrap().as_s().unwrap(),
                    &ziso_timestamp(&timestamp)
                );
                assert_eq!(
                    values.get(":station_locked_ts").unwrap().as_s().unwrap(),
                    &ziso_timestamp(&station.updated_ts)
                );
            }
            _ => unreachable!(),
        }

        let expected_new_station = {
            let mut new_station = station.clone();

            new_station.updated_ts = timestamp;
            new_station.latest_play = Some(latest_play);
            new_station.play_count += 1;

            new_station
        };

        // FIXME: Test track struct update as well
        callback(&mut station, None);

        assert_eq!(station, expected_new_station);
    }
}
