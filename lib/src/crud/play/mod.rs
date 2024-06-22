pub mod models;

use std::sync::Arc;

use anyhow::Result;
use aws_sdk_dynamodb::types::{AttributeValue, Select};
use chrono::{DateTime, Duration, Utc};
use field::field;
use ulid::Ulid;

use crate::crud::shared::models::PaginateKey;
use crate::crud::station::models::StationId;
use crate::crud::Context;
use crate::helpers::truncate_datetime_to_days;
use models::PlayInDB;

pub struct CRUDPlay {
    context: Arc<Context>,
}

impl CRUDPlay {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
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

        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("#pk = :pk AND #sk BETWEEN :start_sk AND :end_sk")
            .expression_attribute_names("#pk", field!(pk @ PlayInDB))
            .expression_attribute_names("#sk", field!(sk @ PlayInDB))
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(PlayInDB::get_pk(station_id, &partition_datetime)),
            )
            .expression_attribute_values(
                ":start_sk",
                AttributeValue::S(PlayInDB::get_sk_prefix() + &start_ulid.to_string()),
            )
            .expression_attribute_values(
                ":end_sk",
                AttributeValue::S(PlayInDB::get_sk_prefix() + &end_ulid.to_string()),
            )
            .select(Select::AllAttributes)
            .limit(limit);

        if let Some(next_key) = next_key {
            // assume no exclusive_start_key if next_key random part is 0
            if next_key.random() != 0 {
                let next_key_datetime: DateTime<Utc> = next_key.datetime().into();
                let play_id = next_key.into();

                query = query
                    .exclusive_start_key(
                        field!(pk @ PlayInDB),
                        AttributeValue::S(PlayInDB::get_pk(station_id, &next_key_datetime)),
                    )
                    .exclusive_start_key(
                        field!(sk @ PlayInDB),
                        AttributeValue::S(PlayInDB::get_sk(play_id)),
                    );
            }
        }

        let resp = query.send().await?;

        let new_next_key = if let Some(last_evaluated_key) = resp.last_evaluated_key {
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

        if let Some(items) = resp.items {
            Ok((serde_dynamo::from_items(items.to_vec())?, new_next_key))
        } else {
            Ok((vec![], new_next_key))
        }
    }
}
