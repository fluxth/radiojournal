use serde::Serialize;

use crate::crud::play::models::PlayId;
use crate::crud::track::models::TrackId;

pub trait Play {
    fn get_title(&self) -> &str;
    fn get_artist(&self) -> &str;
    fn is_song(&self) -> bool;
}

#[derive(Debug, Serialize)]
pub struct AddPlayResult {
    #[serde(flatten)]
    pub add_type: AddPlayType,
    pub play_id: PlayId,
    pub track_id: TrackId,
    pub(super) metadata: AddPlayMetadata,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum AddPlayType {
    ExistingPlay,
    NewPlay,
    NewTrack,
}

impl From<AddPlayTypeInternal> for AddPlayType {
    fn from(value: AddPlayTypeInternal) -> Self {
        match value {
            AddPlayTypeInternal::ExistingPlay { .. } => AddPlayType::ExistingPlay,
            AddPlayTypeInternal::NewPlay { .. } => AddPlayType::NewPlay,
            AddPlayTypeInternal::NewTrack => AddPlayType::NewTrack,
        }
    }
}

#[derive(Debug)]
pub(super) enum AddPlayTypeInternal {
    ExistingPlay { track_id: TrackId, play_id: PlayId },
    NewPlay { track_id: TrackId },
    NewTrack,
}

#[derive(Debug, Serialize)]
pub(super) struct AddPlayMetadata {
    pub title: String,
    pub artist: String,
}
