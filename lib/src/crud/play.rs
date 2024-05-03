use std::sync::Arc;

use anyhow::Result;
use aws_sdk_dynamodb::types::{AttributeValue, Select};
use chrono::{DateTime, Utc};
use ulid::Ulid;

use crate::{
    crud::Context,
    models::{id::StationId, play::PlayInDB, PaginateKey},
};

pub struct CRUDPlay {
    context: Arc<Context>,
}

impl CRUDPlay {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    // todo: traverse play partitions
    pub async fn list_plays(
        &self,
        station_id: StationId,
        limit: i32,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        next_key: Option<Ulid>,
    ) -> Result<(Vec<PlayInDB>, Option<Ulid>)> {
        let play_datetime = if let Some(start) = start {
            start
        } else {
            Utc::now()
        };

        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND sk BETWEEN :start_sk AND :end_sk")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(PlayInDB::get_pk(station_id, &play_datetime)),
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
            .select(Select::AllAttributes)
            .limit(limit);

        if let Some(next_key) = next_key {
            let next_key_datetime: DateTime<Utc> = next_key.datetime().into();
            let play_id = next_key.into();

            query = query
                .exclusive_start_key(
                    "pk",
                    AttributeValue::S(PlayInDB::get_pk(station_id, &next_key_datetime)),
                )
                .exclusive_start_key("sk", AttributeValue::S(PlayInDB::get_sk(play_id)));
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
