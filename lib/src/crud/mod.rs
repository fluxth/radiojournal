use aws_sdk_dynamodb::Client;

pub mod logger;
pub mod play;
pub mod shared;
pub mod station;
pub mod track;

pub struct Context {
    pub(crate) db_client: Client,
    pub(crate) db_table: String,
}

impl Context {
    pub fn new(db_client: Client, db_table: String) -> Self {
        Self {
            db_client,
            db_table,
        }
    }
}
