use std::collections::HashMap;
use std::sync::Arc;

use aws_sdk_dynamodb::error::{BuildError, SdkError};
use aws_sdk_dynamodb::operation::transact_write_items::{
    TransactWriteItemsError, TransactWriteItemsOutput,
};
use aws_sdk_dynamodb::operation::update_item::{UpdateItemError, UpdateItemOutput};
use aws_sdk_dynamodb::types::{
    AttributeValue, Put, TransactWriteItem as DDBTransactWriteItem, Update,
};
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;

use crate::crud::Context;

pub(super) struct DynamoDBProvider {
    context: Arc<Context>,
}

pub(super) struct UpdatePlayInput {
    pub pk: String,
    pub sk: String,
    pub play_id: String,
    pub track_id: String,
    pub update_timestamp: String,
}

#[derive(Debug, PartialEq)]
pub(super) enum TransactWriteItem {
    Put(Put),
    Update(Update),
}

impl DynamoDBProvider {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    pub fn table_name(&self) -> &str {
        &self.context.db_table
    }

    pub async fn update_play(
        &self,
        input: UpdatePlayInput,
    ) -> Result<UpdateItemOutput, SdkError<UpdateItemError, HttpResponse>> {
        self.context
            .db_client
            .update_item()
            .table_name(&self.context.db_table)
            .key("pk", AttributeValue::S(input.pk))
            .key("sk", AttributeValue::S(input.sk))
            .condition_expression("id = :play_id AND track_id = :track_id")
            .update_expression("SET updated_ts = :ts")
            .expression_attribute_values(":play_id", AttributeValue::S(input.play_id))
            .expression_attribute_values(":track_id", AttributeValue::S(input.track_id))
            .expression_attribute_values(":ts", AttributeValue::S(input.update_timestamp))
            .send()
            .await
    }

    pub async fn transact_write_items(
        &self,
        items: impl IntoIterator<Item = TransactWriteItem>,
    ) -> Result<TransactWriteItemsOutput, SdkError<TransactWriteItemsError, HttpResponse>> {
        let mut transaction = self.context.db_client.transact_write_items();

        for item in items.into_iter() {
            transaction = transaction.transact_items(match item {
                TransactWriteItem::Put(put) => DDBTransactWriteItem::builder().put(put).build(),
                TransactWriteItem::Update(update) => {
                    DDBTransactWriteItem::builder().update(update).build()
                }
            });
        }

        transaction.send().await
    }
}

pub fn build_put(
    table_name: &str,
    value: HashMap<String, AttributeValue>,
) -> Result<Put, BuildError> {
    Put::builder()
        .table_name(table_name)
        .set_item(Some(value))
        .build()
}

pub(super) struct BuildTrackUpdateInput {
    pub pk: String,
    pub sk: String,
    pub play_id: String,
    pub update_timestamp: String,
}

pub fn build_track_update(
    table_name: &str,
    input: BuildTrackUpdateInput,
) -> Result<Update, BuildError> {
    Update::builder()
        .table_name(table_name)
        .key("pk", AttributeValue::S(input.pk))
        .key("sk", AttributeValue::S(input.sk))
        .update_expression(
            "SET updated_ts = :ts, latest_play_id = :play_id, play_count = play_count + :inc",
        )
        .expression_attribute_values(":ts", AttributeValue::S(input.update_timestamp))
        .expression_attribute_values(":play_id", AttributeValue::S(input.play_id))
        .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
        .build()
}

#[derive(Clone, Copy)]
pub(super) enum StationUpdateIncrementType {
    Play,
    PlayAndTrack,
}

pub(super) struct BuildStationUpdateInput {
    pub pk: String,
    pub sk: String,
    pub increment: StationUpdateIncrementType,
    pub latest_play: HashMap<String, AttributeValue>,
    pub first_play_id: Option<String>,
    pub update_timestamp: String,
    pub locked_timestamp: Option<String>,
}

pub fn build_station_update(
    table_name: &str,
    input: BuildStationUpdateInput,
) -> Result<Update, BuildError> {
    let mut update_builder = Update::builder()
        .table_name(table_name)
        .key("pk", AttributeValue::S(input.pk))
        .key("sk", AttributeValue::S(input.sk));

    let mut update_expression_parts = vec!["updated_ts = :ts", "latest_play = :latest_play"];
    update_builder = update_builder
        .expression_attribute_values(":ts", AttributeValue::S(input.update_timestamp))
        .expression_attribute_values(":latest_play", AttributeValue::M(input.latest_play));

    update_expression_parts.extend_from_slice(match input.increment {
        StationUpdateIncrementType::Play => &["play_count = play_count + :inc"],
        StationUpdateIncrementType::PlayAndTrack => &[
            "play_count = play_count + :inc",
            "track_count = track_count + :inc",
        ],
    });
    update_builder =
        update_builder.expression_attribute_values(":inc", AttributeValue::N("1".to_string()));

    let mut condition_expression_parts = vec![];
    if let Some(play_id) = input.first_play_id {
        update_expression_parts.push("first_play_id = :play_id");
        condition_expression_parts.push("first_play_id = :null");
        update_builder = update_builder
            .expression_attribute_values(":play_id", AttributeValue::S(play_id.to_string()))
            .expression_attribute_values(":null", AttributeValue::Null(true));
    }

    if let Some(locked_timestamp) = input.locked_timestamp {
        condition_expression_parts.push("updated_ts = :station_locked_ts");
        update_builder = update_builder
            .expression_attribute_values(":station_locked_ts", AttributeValue::S(locked_timestamp));
    }

    let update_expression = format!("SET {}", update_expression_parts.join(", "));
    update_builder = update_builder.update_expression(update_expression);

    if !condition_expression_parts.is_empty() {
        let condition_expression = condition_expression_parts.join(" AND ");
        update_builder = update_builder.condition_expression(condition_expression);
    }

    update_builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[test]
    fn test_build_put_success() {
        let put = build_put(
            "tablename",
            HashMap::from([
                ("pk".to_owned(), AttributeValue::S("pkvalue".to_owned())),
                ("sk".to_owned(), AttributeValue::S("skvalue".to_owned())),
            ]),
        )
        .unwrap();

        assert_eq!(
            put,
            Put::builder()
                .table_name("tablename")
                .item("pk", AttributeValue::S("pkvalue".to_owned()))
                .item("sk", AttributeValue::S("skvalue".to_owned()))
                .build()
                .unwrap()
        )
    }

    #[test]
    fn test_build_track_update_success() {
        let update = build_track_update(
            "tablename",
            BuildTrackUpdateInput {
                pk: "pkvalue".to_owned(),
                sk: "skvalue".to_owned(),
                play_id: "playid".to_owned(),
                update_timestamp: "12345".to_owned(),
            },
        )
        .unwrap();

        assert_eq!(
            update,
            Update::builder()
                .table_name("tablename")
                .key("pk", AttributeValue::S("pkvalue".to_owned()))
                .key("sk", AttributeValue::S("skvalue".to_owned()))
                .update_expression(
                    "SET updated_ts = :ts, latest_play_id = :play_id, play_count = play_count + :inc",
                )
                .expression_attribute_values(":ts", AttributeValue::S("12345".to_owned()))
                .expression_attribute_values(":play_id", AttributeValue::S("playid".to_owned()))
                .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
                .build()
                .unwrap()
        );
    }

    fn base_build_station_update_input() -> BuildStationUpdateInput {
        BuildStationUpdateInput {
            pk: "pkvalue".to_owned(),
            sk: "skvalue".to_owned(),
            increment: StationUpdateIncrementType::Play,
            latest_play: HashMap::from_iter([("a".to_owned(), AttributeValue::S("b".to_owned()))]),
            first_play_id: Some("firstplayid".to_owned()),
            update_timestamp: "12345".to_owned(),
            locked_timestamp: Some("67890".to_owned()),
        }
    }

    #[test]
    fn test_build_station_update_base() {
        assert_eq!(
            build_station_update("tablename", base_build_station_update_input()).unwrap(),
            Update::builder()
                .table_name("tablename")
                .key("pk", AttributeValue::S("pkvalue".to_owned()))
                .key("sk", AttributeValue::S("skvalue".to_owned()))
                .update_expression(
                    "SET updated_ts = :ts, latest_play = :latest_play, play_count = play_count + :inc, first_play_id = :play_id"
                )
                .condition_expression("first_play_id = :null AND updated_ts = :station_locked_ts")
                .expression_attribute_values(":ts", AttributeValue::S("12345".to_owned()))
                .expression_attribute_values(
                    ":latest_play",
                    AttributeValue::M(
                        HashMap::from_iter([
                            ("a".to_owned(), AttributeValue::S("b".to_owned()))
                        ])
                    )
                )
                .expression_attribute_values(":station_locked_ts", AttributeValue::S("67890".to_owned()))
                .expression_attribute_values(":play_id", AttributeValue::S("firstplayid".to_owned()))
                .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
                .expression_attribute_values(":null", AttributeValue::Null(true))
                .build()
                .unwrap()
        );
    }

    #[rstest]
    fn test_build_station_update_matrix(
        #[values(
            StationUpdateIncrementType::Play,
            StationUpdateIncrementType::PlayAndTrack
        )]
        increment: StationUpdateIncrementType,
        #[values(
            None,
            Some("firstplayid".to_owned()),
        )]
        first_play_id: Option<String>,
        #[values(
            None,
            Some("lockedtimestamp".to_owned()),
        )]
        locked_timestamp: Option<String>,
    ) {
        let update = build_station_update(
            "tablename",
            BuildStationUpdateInput {
                increment,
                first_play_id: first_play_id.clone(),
                locked_timestamp: locked_timestamp.clone(),
                ..base_build_station_update_input()
            },
        )
        .unwrap();

        assert!(update.update_expression().starts_with("SET "));
        let update_expression_parts: Vec<&str> = update
            .update_expression()
            .trim_start_matches("SET ")
            .split(", ")
            .collect();

        let condition_expression_parts: Vec<&str> = update
            .condition_expression()
            .unwrap_or("")
            .split(" AND ")
            .collect();

        let expression_attribute_values = update.expression_attribute_values().unwrap();

        match increment {
            StationUpdateIncrementType::Play => {
                assert!(update_expression_parts.contains(&"play_count = play_count + :inc"));
            }
            StationUpdateIncrementType::PlayAndTrack => {
                assert!(update_expression_parts.contains(&"play_count = play_count + :inc"));
                assert!(update_expression_parts.contains(&"track_count = track_count + :inc"));
            }
        }

        assert_eq!(
            expression_attribute_values.get(":inc").unwrap(),
            &AttributeValue::N("1".to_owned())
        );

        match first_play_id {
            Some(play_id) => {
                assert!(update_expression_parts.contains(&"first_play_id = :play_id"));
                assert!(condition_expression_parts.contains(&"first_play_id = :null"));
                assert_eq!(
                    expression_attribute_values.get(":play_id").unwrap(),
                    &AttributeValue::S(play_id),
                );
                assert_eq!(
                    expression_attribute_values.get(":null").unwrap(),
                    &AttributeValue::Null(true),
                );
            }
            None => {
                assert!(!update_expression_parts.contains(&"first_play_id = :play_id"));
                assert!(!condition_expression_parts.contains(&"first_play_id = :null"));
                assert!(!expression_attribute_values.contains_key(":play_id"));
                assert!(!expression_attribute_values.contains_key(":null"));
            }
        }

        match locked_timestamp {
            Some(locked_timestamp) => {
                assert!(condition_expression_parts.contains(&"updated_ts = :station_locked_ts"));
                assert_eq!(
                    expression_attribute_values
                        .get(":station_locked_ts")
                        .unwrap(),
                    &AttributeValue::S(locked_timestamp),
                );
            }
            None => {
                assert!(!condition_expression_parts.contains(&"updated_ts = :station_locked_ts"));
                assert!(!expression_attribute_values.contains_key(":station_locked_ts"));
            }
        }
    }
}
