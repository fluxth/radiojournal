data "aws_iam_policy_document" "assume_role" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRoleWithWebIdentity"]

    principals {
      type        = "Federated"
      identifiers = ["arn:aws:iam::${data.aws_caller_identity.current.account_id}:oidc-provider/token.actions.githubusercontent.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:aud"
      values   = ["sts.amazonaws.com"]
    }

    condition {
      test     = "StringLike"
      variable = "token.actions.githubusercontent.com:sub"
      values   = ["repo:${var.allowed_repo}:ref:refs/tags/*"]
    }
  }

  dynamic "statement" {
    for_each = length(var.allowed_assume_role) > 0 ? [1] : []
    content {
      effect  = "Allow"
      actions = ["sts:AssumeRole", "sts:TagSession"]

      principals {
        type        = "AWS"
        identifiers = var.allowed_assume_role
      }
    }
  }
}

data "aws_iam_policy_document" "lambda_permissions" {
  statement {
    effect = "Allow"
    actions = [
      "lambda:GetFunction",
      "lambda:UpdateFunctionCode",
      "lambda:GetFunctionConfiguration",
      "lambda:PublishVersion",
    ]
    resources = [
      var.api_function_arn,
      var.logger_function_arn,
    ]
  }

  statement {
    effect = "Allow"
    actions = [
      "lambda:InvokeFunction",
      "lambda:UpdateAlias",
    ]
    resources = [
      "${var.api_function_arn}:*",
      "${var.logger_function_arn}:*",
    ]
  }
}

data "aws_iam_policy_document" "s3_permissions" {
  statement {
    effect    = "Allow"
    actions   = ["s3:ListBucket"]
    resources = [var.frontend_bucket_arn]
  }

  statement {
    effect = "Allow"
    actions = [
      "s3:GetObject",
      "s3:PutObject",
      "s3:DeleteObject",
    ]
    resources = ["${var.frontend_bucket_arn}/*"]
  }
}

resource "aws_iam_role_policy" "lambda_permissions" {
  name   = "LambdaPermissions"
  role   = aws_iam_role.this.id
  policy = data.aws_iam_policy_document.lambda_permissions.json
}

resource "aws_iam_role_policy" "s3_permissions" {
  name   = "S3Permissions"
  role   = aws_iam_role.this.id
  policy = data.aws_iam_policy_document.s3_permissions.json
}
