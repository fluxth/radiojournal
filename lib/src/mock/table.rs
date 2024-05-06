use std::sync::Arc;

use anyhow::Result;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection,
    ProjectionType, ScalarAttributeType,
};
use tracing::warn;

use crate::crud::Context;

pub async fn delete_then_create_table(context: Arc<Context>) -> Result<()> {
    if context
        .db_client
        .delete_table()
        .table_name(&context.db_table)
        .send()
        .await
        .is_err()
    {
        warn!(
            "Table {} does not exist, skipping delete.",
            context.db_table
        );
    }

    let ks_pk = KeySchemaElement::builder()
        .attribute_name("pk")
        .key_type(KeyType::Hash)
        .build()?;

    let ks_sk = KeySchemaElement::builder()
        .attribute_name("sk")
        .key_type(KeyType::Range)
        .build()?;

    let ad_pk = AttributeDefinition::builder()
        .attribute_name("pk")
        .attribute_type(ScalarAttributeType::S)
        .build()?;

    let ad_sk = AttributeDefinition::builder()
        .attribute_name("sk")
        .attribute_type(ScalarAttributeType::S)
        .build()?;

    let ad_gsi1pk = AttributeDefinition::builder()
        .attribute_name("gsi1pk")
        .attribute_type(ScalarAttributeType::S)
        .build()?;

    let gsi1_ks_gsi1pk = KeySchemaElement::builder()
        .attribute_name("gsi1pk")
        .key_type(KeyType::Hash)
        .build()?;

    let gsi1_projection = Projection::builder()
        .projection_type(ProjectionType::Include)
        .non_key_attributes("id")
        .non_key_attributes("track_id")
        .build();

    let gsi1 = GlobalSecondaryIndex::builder()
        .index_name("gsi1")
        .key_schema(gsi1_ks_gsi1pk)
        .key_schema(ks_sk.clone())
        .projection(gsi1_projection)
        .build()?;

    context
        .db_client
        .create_table()
        .table_name(&context.db_table)
        .key_schema(ks_pk)
        .key_schema(ks_sk)
        .attribute_definitions(ad_pk)
        .attribute_definitions(ad_sk)
        .attribute_definitions(ad_gsi1pk)
        .global_secondary_indexes(gsi1)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    Ok(())
}
