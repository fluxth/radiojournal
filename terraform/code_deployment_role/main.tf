resource "aws_iam_role" "this" {
  name               = var.role_name
  description        = "Code Deployment Role for GitHub Actions"
  assume_role_policy = data.aws_iam_policy_document.assume_role.json
}
