use std::sync::Arc;

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::update_item::{UpdateItemError, UpdateItemOutput};
use aws_sdk_dynamodb::types::AttributeValue;
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

impl DynamoDBProvider {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
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
}
