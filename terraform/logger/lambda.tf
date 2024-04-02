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

  logging_config {
    log_format = "JSON"
  }

  layers = [
    "arn:aws:lambda:ap-southeast-1:580247275435:layer:LambdaInsightsExtension-Arm64:16"
  ]
}

resource "aws_lambda_function_event_invoke_config" "this" {
  function_name                = aws_lambda_function.this.function_name
  maximum_event_age_in_seconds = 60
  maximum_retry_attempts       = 1
}
