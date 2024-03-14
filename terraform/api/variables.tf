variable "name" {
  type = string
}

variable "lambda_zip_path" {
  type = string
}

variable "lambda_zip_hash" {
  type = string
}

variable "db_table_name" {
  type = string
}

variable "allowed_cors_domains" {
  type = list(string)
}
