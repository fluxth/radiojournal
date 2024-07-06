use std::collections::HashMap;
use std::sync::Arc;

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::batch_get_item::{BatchGetItemError, BatchGetItemOutput};
use aws_sdk_dynamodb::operation::get_item::{GetItemError, GetItemOutput};
use aws_sdk_dynamodb::operation::query::{QueryError, QueryOutput};
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, Select};
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;

use crate::crud::Context;

pub(super) struct DynamoDBProvider {
    context: Arc<Context>,
}

pub(super) enum ProjectedFields {
    All,
    Some(&'static [&'static str]),
}

pub(super) struct ExclusiveStartKey {
    pub pk: String,
    pub sk: String,
}

pub(super) struct Gsi1ExclusiveStartKey {
    pub gsi1pk: String,
    pub sk: String,
    pub pk: String,
}

pub(super) struct GetItemInput {
    pub pk: String,
    pub sk: String,
}

pub(super) struct GetItemConfig {
    pub consistent_read: bool,
    pub projected_fields: ProjectedFields,
}

pub(super) struct QueryPrefixInput {
    pub pk: String,
    pub sk_prefix: String,
    pub scan_forward: bool,
    pub exclusive_start_key: Option<ExclusiveStartKey>,
}

pub(super) struct QueryPrefixConfig {
    pub limit: i32,
    pub projected_fields: ProjectedFields,
}

pub(super) struct QueryPrefixGsi1Input {
    pub gsi1pk: String,
    pub sk_prefix: String,
    pub pk_prefix: Option<String>,
    pub scan_forward: bool,
    pub exclusive_start_key: Option<Gsi1ExclusiveStartKey>,
}

pub(super) struct QueryPrefixGsi1Config {
    pub limit: i32,
}

pub(super) struct BatchGetItemKey {
    pub pk: String,
    pub sk: String,
}

pub(super) struct BatchGetItemInput<I>
where
    I: Iterator<Item = BatchGetItemKey>,
{
    pub keys: I,
}

pub(super) struct BatchGetItemConfig {
    pub projected_fields: ProjectedFields,
}

impl DynamoDBProvider {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    pub fn table_name(&self) -> &str {
        &self.context.db_table
    }

    pub async fn get_item(
        &self,
        input: GetItemInput,
        config: GetItemConfig,
    ) -> Result<GetItemOutput, SdkError<GetItemError, HttpResponse>> {
        let mut get_item = self
            .context
            .db_client
            .get_item()
            .table_name(&self.context.db_table)
            .key("pk", AttributeValue::S(input.pk))
            .key("sk", AttributeValue::S(input.sk))
            .consistent_read(config.consistent_read);

        if let ProjectedFields::Some(projected_fields) = config.projected_fields {
            get_item = get_item.projection_expression(projected_fields.join(", "));
        }

        get_item.send().await
    }

    pub async fn query_prefix(
        &self,
        input: QueryPrefixInput,
        config: QueryPrefixConfig,
    ) -> Result<QueryOutput, SdkError<QueryError, HttpResponse>> {
        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk_prefix)")
            .expression_attribute_values(":pk", AttributeValue::S(input.pk))
            .expression_attribute_values(":sk_prefix", AttributeValue::S(input.sk_prefix))
            .scan_index_forward(input.scan_forward)
            .limit(config.limit);

        query = match config.projected_fields {
            ProjectedFields::All => query.select(Select::AllAttributes),
            ProjectedFields::Some(projected_fields) => query
                .select(Select::SpecificAttributes)
                .projection_expression(projected_fields.join(", ")),
        };

        if let Some(exclusive_start_key) = input.exclusive_start_key {
            query = query
                .exclusive_start_key("pk", AttributeValue::S(exclusive_start_key.pk))
                .exclusive_start_key("sk", AttributeValue::S(exclusive_start_key.sk));
        }

        query.send().await
    }

    pub async fn query_prefix_gsi1(
        &self,
        input: QueryPrefixGsi1Input,
        config: QueryPrefixGsi1Config,
    ) -> Result<QueryOutput, SdkError<QueryError, HttpResponse>> {
        let mut query = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .index_name("gsi1")
            .key_condition_expression("gsi1pk = :gsi1pk AND begins_with(sk, :sk_prefix)")
            .expression_attribute_values(":gsi1pk", AttributeValue::S(input.gsi1pk))
            .expression_attribute_values(":sk_prefix", AttributeValue::S(input.sk_prefix))
            .scan_index_forward(input.scan_forward)
            .limit(config.limit);

        if let Some(pk_prefix) = input.pk_prefix {
            query = query
                .filter_expression("begins_with(pk, :pk_prefix)")
                .expression_attribute_values(":pk_prefix", AttributeValue::S(pk_prefix))
        }

        if let Some(exclusive_start_key) = input.exclusive_start_key {
            query = query
                .exclusive_start_key("gsi1pk", AttributeValue::S(exclusive_start_key.gsi1pk))
                .exclusive_start_key("sk", AttributeValue::S(exclusive_start_key.sk))
                .exclusive_start_key("pk", AttributeValue::S(exclusive_start_key.pk));
        }

        query.send().await
    }

    pub async fn batch_get_item<I>(
        &self,
        input: BatchGetItemInput<I>,
        config: BatchGetItemConfig,
    ) -> Result<BatchGetItemOutput, SdkError<BatchGetItemError, HttpResponse>>
    where
        I: Iterator<Item = BatchGetItemKey>,
    {
        let mut request_keys = KeysAndAttributes::builder();

        if let ProjectedFields::Some(projected_fields) = config.projected_fields {
            request_keys = request_keys.projection_expression(projected_fields.join(", "));
        }

        // FIXME: do multiple batches if key count > 100
        for key in input.keys {
            request_keys = request_keys.keys(HashMap::from([
                ("pk".to_owned(), AttributeValue::S(key.pk)),
                ("sk".to_owned(), AttributeValue::S(key.sk)),
            ]))
        }

        self.context
            .db_client
            .batch_get_item()
            .request_items(&self.context.db_table, request_keys.build()?)
            .send()
            .await
    }
}
