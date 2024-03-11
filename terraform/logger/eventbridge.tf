resource "aws_cloudwatch_event_rule" "this" {
  name        = var.name
  description = "Invokes ${var.name} lambda function every minute"

  schedule_expression = "cron(* * * * ? *)"
}

resource "aws_cloudwatch_event_target" "this" {
  rule = aws_cloudwatch_event_rule.this.id
  arn  = aws_lambda_function.this.arn
}
