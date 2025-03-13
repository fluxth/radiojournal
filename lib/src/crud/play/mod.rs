pub mod models;
mod provider;

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use ulid::Ulid;

use crate::crud::Context;
use crate::crud::shared::models::PaginateKey;
use crate::crud::station::models::StationId;
use crate::helpers::truncate_datetime_to_days;
use models::PlayInDB;
use provider::{DynamoDBProvider, ExclusiveStartKey, QueryRangeConfig, QueryRangeInput};

pub struct CRUDPlay {
    provider: DynamoDBProvider,
}

impl CRUDPlay {
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            provider: DynamoDBProvider::new(context),
        }
    }

    pub async fn list_plays(
        &self,
        station_id: StationId,
        limit: i32,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        next_key: Option<Ulid>,
    ) -> Result<(Vec<PlayInDB>, Option<Ulid>)> {
        let partition_datetime = if let Some(next_key) = next_key {
            next_key.datetime().into()
        } else {
            start
        };

        let start_ulid = Ulid::from_parts(start.timestamp_millis().try_into()?, 0);
        let end_ulid = Ulid::from_parts(end.timestamp_millis().try_into()?, u128::MAX);

        let exclusive_start_key = if let Some(next_key) = next_key {
            // assume no exclusive_start_key if next_key random part is 0
            if next_key.random() != 0 {
                let next_key_datetime: DateTime<Utc> = next_key.datetime().into();
                let play_id = next_key.into();

                Some(ExclusiveStartKey {
                    pk: PlayInDB::get_pk(station_id, &next_key_datetime),
                    sk: PlayInDB::get_sk(play_id),
                })
            } else {
                None
            }
        } else {
            None
        };

        let query_result = self
            .provider
            .query_range(
                QueryRangeInput {
                    pk: PlayInDB::get_pk(station_id, &partition_datetime),
                    start_sk: PlayInDB::get_sk_prefix() + &start_ulid.to_string(),
                    end_sk: PlayInDB::get_sk_prefix() + &end_ulid.to_string(),
                    exclusive_start_key,
                },
                QueryRangeConfig { limit },
            )
            .await?;

        let new_next_key = if let Some(last_evaluated_key) = query_result.last_evaluated_key {
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
        } else if !PlayInDB::is_same_partition(&partition_datetime, &end_ulid.datetime().into()) {
            let next_partition_datetime = truncate_datetime_to_days(partition_datetime)
                .expect("truncate partition datetime to days")
                + Duration::days(1);

            Some(Ulid::from_parts(
                truncate_datetime_to_days(next_partition_datetime)
                    .expect("truncate partition datetime to days")
                    .timestamp_millis()
                    .try_into()?,
                0,
            ))
        } else {
            None
        };

        if let Some(items) = query_result.items {
            Ok((serde_dynamo::from_items(items.to_vec())?, new_next_key))
        } else {
            Ok((vec![], new_next_key))
        }
    }
}
