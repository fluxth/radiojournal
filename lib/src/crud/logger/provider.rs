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
