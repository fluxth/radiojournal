use std::ops::Deref;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
