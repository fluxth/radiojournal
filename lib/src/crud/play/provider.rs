use std::sync::Arc;

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::query::{QueryError, QueryOutput};
use aws_sdk_dynamodb::types::{AttributeValue, Select};
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;

use crate::crud::Context;

pub(super) struct DynamoDBProvider {
    context: Arc<Context>,
}

pub(super) struct ExclusiveStartKey {
    pub pk: String,
    pub sk: String,
}

pub(super) struct QueryRangeInput {
    pub pk: String,
    pub start_sk: String,
    pub end_sk: String,
    pub exclusive_start_key: Option<ExclusiveStartKey>,
}

pub(super) struct QueryRangeConfig {
    pub limit: i32,
}

impl DynamoDBProvider {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    pub async fn query_range(
        &self,
        input: QueryRangeInput,
        config: QueryRangeConfig,
    ) -> Result<QueryOutput, SdkError<QueryError, HttpResponse>> {
        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND sk BETWEEN :start_sk AND :end_sk")
            .expression_attribute_values(":pk", AttributeValue::S(input.pk))
            .expression_attribute_values(":start_sk", AttributeValue::S(input.start_sk))
            .expression_attribute_values(":end_sk", AttributeValue::S(input.end_sk))
            .select(Select::AllAttributes)
            .limit(config.limit);

        if let Some(exclusive_start_key) = input.exclusive_start_key {
            query = query
                .exclusive_start_key("pk", AttributeValue::S(exclusive_start_key.pk))
                .exclusive_start_key("sk", AttributeValue::S(exclusive_start_key.sk));
        }

        query.send().await
    }
}
