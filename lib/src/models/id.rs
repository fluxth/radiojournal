use std::ops::Deref;

use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct StationId(pub Ulid);

impl From<Ulid> for StationId {
    fn from(val: Ulid) -> Self {
        Self(val)
    }
}

impl Deref for StationId {
    type Target = Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct TrackId(pub Ulid);

impl From<Ulid> for TrackId {
    fn from(val: Ulid) -> Self {
        Self(val)
    }
}

impl Deref for TrackId {
    type Target = Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PlayId(pub Ulid);

impl From<Ulid> for PlayId {
    fn from(val: Ulid) -> Self {
        Self(val)
    }
}

impl Deref for PlayId {
    type Target = Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
