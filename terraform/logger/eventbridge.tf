resource "aws_scheduler_schedule" "scheduler" {
  name       = "${var.name}-scheduler"
  group_name = "default"

  state = var.enabled ? "ENABLED" : "DISABLED"

  schedule_expression = "rate(1 minute)"

  flexible_time_window {
    mode                      = "FLEXIBLE"
    maximum_window_in_minutes = 1
  }

  target {
    arn      = aws_lambda_function.this.arn
    role_arn = aws_iam_role.scheduler.arn

    input = jsonencode({})

    retry_policy {
      maximum_event_age_in_seconds = 60
      maximum_retry_attempts       = 10
    }
  }
}
