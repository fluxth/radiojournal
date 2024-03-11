resource "aws_lambda_function" "this" {
  function_name = var.name
  description   = "Logger lambda for radiojournal"
  role          = aws_iam_role.lambda.arn

  filename         = var.lambda_zip_path
  source_code_hash = var.lambda_zip_hash
  handler          = "bootstrap"

  architectures = ["arm64"]
  runtime       = "provided.al2023"

  memory_size = 128
  timeout     = 10

  environment {
    variables = {
      DB_TABLE_NAME = var.db_table_name
    }
  }
}

resource "aws_lambda_permission" "eventbridge" {
  function_name = aws_lambda_function.this.function_name
  statement_id  = "EventBridgePermissions"
  action        = "lambda:InvokeFunction"
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.this.arn
}
