variable "role_name" {
  type = string
}

variable "allowed_repo" {
  type        = string
  description = "GitHub repository in 'owner/repo' format"
}

variable "allowed_assume_role" {
  type        = list(string)
  description = "Expected format 'arn:aws:sts::123456789012:assumed-role/user/tag'"
  default     = []
}

variable "api_function_arn" {
  type        = string
  description = "ARN of the API lambda function"
}

variable "logger_function_arn" {
  type        = string
  description = "ARN of the logger lambda function"
}

variable "frontend_bucket_arn" {
  type        = string
  description = "ARN of the S3 bucket hosting the frontend"
}
