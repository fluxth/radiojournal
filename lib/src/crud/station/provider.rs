use std::collections::HashMap;
use std::sync::Arc;

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::get_item::{GetItemError, GetItemOutput};
use aws_sdk_dynamodb::operation::put_item::{PutItemError, PutItemOutput};
use aws_sdk_dynamodb::operation::query::{QueryError, QueryOutput};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;

use crate::crud::Context;

pub(super) struct DynamoDBProvider {
    context: Arc<Context>,
}

pub(super) struct GetItemInput {
    pub pk: String,
    pub sk: String,
}

pub(super) struct PutItemInput {
    pub item: HashMap<String, AttributeValue>,
}

pub(super) struct QueryPrefixInput {
    pub pk: String,
    pub sk_prefix: String,
}

pub(super) struct QueryPrefixConfig {
    pub limit: i32,
}

impl DynamoDBProvider {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    pub async fn get_item(
        &self,
        input: GetItemInput,
    ) -> Result<GetItemOutput, SdkError<GetItemError, HttpResponse>> {
        self.context
            .db_client
            .get_item()
            .table_name(&self.context.db_table)
            .key("pk", AttributeValue::S(input.pk))
            .key("sk", AttributeValue::S(input.sk))
            .send()
            .await
    }

    pub async fn put_item(
        &self,
        input: PutItemInput,
    ) -> Result<PutItemOutput, SdkError<PutItemError, HttpResponse>> {
        self.context
            .db_client
            .put_item()
            .table_name(&self.context.db_table)
            .set_item(Some(input.item))
            .send()
            .await
    }

    pub async fn query_prefix(
        &self,
        input: QueryPrefixInput,
        config: QueryPrefixConfig,
    ) -> Result<QueryOutput, SdkError<QueryError, HttpResponse>> {
        self.context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk_prefix)")
            .expression_attribute_values(":pk", AttributeValue::S(input.pk))
            .expression_attribute_values(":sk_prefix", AttributeValue::S(input.sk_prefix))
            .limit(config.limit)
            .send()
            .await
    }
}
