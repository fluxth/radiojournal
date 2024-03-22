plugin "terraform" {
    enabled = true
    version = "0.6.0"
    source  = "github.com/terraform-linters/tflint-ruleset-terraform"
}

plugin "aws" {
    enabled = true
    version = "0.30.0"
    source  = "github.com/terraform-linters/tflint-ruleset-aws"
}

rule "terraform_documented_variables" {
    enabled = false
}

rule "terraform_documented_outputs" {
    enabled = false
}