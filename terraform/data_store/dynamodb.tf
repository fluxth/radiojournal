resource "aws_dynamodb_table" "this" {
  name         = var.name
  billing_mode = "PAY_PER_REQUEST"

  hash_key  = "pk"
  range_key = "sk"

  attribute {
    name = "pk"
    type = "S"
  }

  attribute {
    name = "sk"
    type = "S"
  }

  attribute {
    name = "gsi1pk"
    type = "S"
  }

  global_secondary_index {
    name               = "gsi1"
    hash_key           = "gsi1pk"
    range_key          = "sk"
    projection_type    = "INCLUDE"
    non_key_attributes = ["id", "track_id"]
  }
}
