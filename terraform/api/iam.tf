data "aws_iam_policy_document" "lambda_assume_role" {
  statement {
    effect = "Allow"

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }

    actions = ["sts:AssumeRole"]
  }
}

data "aws_iam_policy_document" "dynamodb_access" {
  statement {
    actions = [
      "dynamodb:Query",
      "dynamodb:GetItem",
      "dynamodb:BatchGetItem",
    ]
    resources = [
      "arn:aws:dynamodb:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:table/${var.db_table_name}",
      "arn:aws:dynamodb:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:table/${var.db_table_name}/index/*",
    ]
  }
}

resource "aws_iam_role" "lambda" {
  name               = var.name
  assume_role_policy = data.aws_iam_policy_document.lambda_assume_role.json

  inline_policy {
    name   = "DynamoDBAccess"
    policy = data.aws_iam_policy_document.dynamodb_access.json
  }
}

resource "aws_iam_role_policy_attachment" "lambda_basic_execution_role" {
  role       = aws_iam_role.lambda.id
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}
