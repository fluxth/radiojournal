pub mod models;
mod provider;

use std::sync::Arc;

use anyhow::Result;
use aws_sdk_dynamodb::types::TransactWriteItem;
use chrono::{DateTime, Utc};

use crate::crud::play::models::{PlayId, PlayInDB};
use crate::crud::station::models::{LatestPlay, StationId, StationInDB};
use crate::crud::track::models::{TrackId, TrackInDB, TrackMetadataCreateInDB};
use crate::crud::track::CRUDTrack;
use crate::crud::Context;
use crate::helpers::ziso_timestamp;
use models::{AddPlayMetadata, AddPlayResult, AddPlayTypeInternal, Play};
use provider::{
    build_put, build_station_update, build_track_update, BuildStationUpdateInput,
    BuildTrackUpdateInput, DynamoDBProvider, StationUpdateIncrementType, UpdatePlayInput,
};

pub struct CRUDLogger {
    context: Arc<Context>, // FIXME: Remove this
    provider: DynamoDBProvider,
    crud_track: CRUDTrack,
}

impl CRUDLogger {
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            crud_track: CRUDTrack::new(context.clone()),
            provider: DynamoDBProvider::new(context.clone()),
            context,
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
        let play_id = play.id;
        let track_id = play.track_id;

        let latest_play = LatestPlay {
            id: play_id,
            track_id,
            artist: artist.to_owned(),
            title: title.to_owned(),
        };

        let now = Utc::now();

        let play_put = build_put(&self.context.db_table, serde_dynamo::to_item(play)?)?;

        let track_update = build_track_update(
            &self.context.db_table,
            BuildTrackUpdateInput {
                pk: TrackInDB::get_pk(station.id),
                sk: TrackInDB::get_sk(track_id),
                play_id: play_id.to_string(),
                update_timestamp: ziso_timestamp(&now),
            },
        )?;

        // update station with latest play
        let station_update = build_station_update(
            &self.context.db_table,
            BuildStationUpdateInput {
                pk: StationInDB::get_pk(),
                sk: StationInDB::get_sk(station.id),
                increment: StationUpdateIncrementType::Play,
                latest_play: serde_dynamo::to_item(latest_play.clone())?,
                first_play_id: None, // first play will always create new track
                update_timestamp: ziso_timestamp(&now),
                locked_timestamp: Some(ziso_timestamp(&station.updated_ts)),
            },
        )?;

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

        station.updated_ts = now;
        station.latest_play = Some(latest_play);
        station.play_count += 1;

        Ok(())
    }

    async fn add_play_with_new_track(
        &self,
        station: &mut StationInDB,
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

        let now = Utc::now();

        let track_put = build_put(&self.context.db_table, serde_dynamo::to_item(track)?)?;
        let track_metadata_put = build_put(
            &self.context.db_table,
            serde_dynamo::to_item(track_metadata)?,
        )?;
        let play_put = build_put(&self.context.db_table, serde_dynamo::to_item(play)?)?;

        // update station with latest play and track
        let station_update = build_station_update(
            &self.context.db_table,
            BuildStationUpdateInput {
                pk: StationInDB::get_pk(),
                sk: StationInDB::get_sk(station.id),
                increment: StationUpdateIncrementType::PlayAndTrack,
                latest_play: serde_dynamo::to_item(latest_play.clone())?,
                first_play_id: match station.first_play_id {
                    None => Some(play_id.to_string()),
                    Some(_) => None,
                },
                update_timestamp: ziso_timestamp(&now),
                locked_timestamp: Some(ziso_timestamp(&station.updated_ts)),
            },
        )?;

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

        station.updated_ts = now;
        station.latest_play = Some(latest_play);
        station.play_count += 1;
        station.track_count += 1;
        if station.first_play_id.is_none() {
            station.first_play_id = Some(play_id)
        }

        Ok(())
    }
}
