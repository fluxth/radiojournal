pub mod id;
pub mod play;
pub mod station;
pub mod track;

use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct PaginateKey {
    pub(crate) pk: String,
    pub(crate) sk: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct Gsi1PaginateKey {
    pub(crate) pk: String,
    pub(crate) gsi1pk: String,
    pub(crate) sk: String,
}
