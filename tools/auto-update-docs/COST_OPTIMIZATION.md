# Cost Optimization Guide

This document explains strategies to reduce costs when using the auto-update-docs tool.

## Cost Comparison: Anthropic API vs AWS Bedrock

### Anthropic API (Direct)

**Pricing** (as of 2025):
- Claude 3.5 Sonnet: $3/M input tokens, $15/M output tokens
- Claude Opus: $15/M input tokens, $75/M output tokens

**Pros**:
- Simple setup (just API key)
- No AWS account needed
- Latest models immediately available

**Cons**:
- Higher cost at scale
- No volume discounts
- External API dependency

**Estimated monthly cost** (nightly runs):
- Input: ~30 runs × 100k tokens × $3/M = **$9/month**
- Output: ~30 runs × 25k tokens × $15/M = **$11.25/month**
- **Total: ~$20-25/month**

### AWS Bedrock (Recommended for Cost Savings!)

**Pricing** (as of 2025):
- **On-Demand**: Same as Anthropic API ($3/M input, $15/M output)
- **Provisioned Throughput**: Up to **50% cheaper** with commitment
- **Batch Mode**: Coming soon, will be even cheaper

**Pros**:
- **Lower cost** with provisioned throughput
- **AWS integration** (IAM, VPC, CloudWatch)
- **No API key management** (uses AWS credentials)
- **Better compliance** (data stays in AWS)
- Volume discounts available

**Cons**:
- Requires AWS account setup
- AWS CLI installation needed
- Slightly more complex initial setup

**Estimated monthly cost** (nightly runs):

**On-Demand** (same as Anthropic):
- Total: ~$20-25/month

**Provisioned Throughput** (1-year commitment):
- Total: ~**$10-12/month** (50% savings!)

**Cost Savings**: Up to **$150/year** with Bedrock Provisioned Throughput

## Strategy 1: Use AWS Bedrock (50% Cost Reduction)

### Setup AWS Bedrock

1. **Install AWS CLI**:
   ```bash
   # macOS
   brew install awscli

   # Linux
   curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
   unzip awscliv2.zip
   sudo ./aws/install

   # Windows
   # Download from https://aws.amazon.com/cli/
   ```

2. **Configure AWS credentials**:
   ```bash
   aws configure
   # Enter:
   # - AWS Access Key ID
   # - AWS Secret Access Key
   # - Default region: us-east-1 (or your preferred region)
   # - Default output format: json
   ```

3. **Enable Bedrock models**:
   - Go to AWS Bedrock console
   - Navigate to "Model access"
   - Request access to Anthropic Claude models
   - Wait for approval (usually instant)

4. **Use Bedrock in the tool**:
   ```bash
   cargo run -p auto-update-docs -- \
     --use-bedrock \
     --aws-region us-east-1
   ```

### Cost Comparison Example

**Scenario**: Update docs nightly for 30 days

| Mode | Input Tokens | Output Tokens | Cost |
|------|-------------|---------------|------|
| Anthropic API | 3M | 750k | $9 + $11.25 = **$20.25** |
| Bedrock On-Demand | 3M | 750k | $9 + $11.25 = **$20.25** |
| Bedrock Provisioned | 3M | 750k | **~$10** |

**Savings**: $10.25/month = **$123/year**

## Strategy 2: Smart Triggers (75% Cost Reduction)

Instead of running nightly, only run when significant changes occur.

### Use the Smart Workflow

Replace `.github/workflows/auto-update-docs.yml` with `.github/workflows/auto-update-docs-smart.yml`:

```yaml
# Only runs when:
# 1. Code changes pushed to main
# 2. Weekly on Sundays
# 3. Manual trigger
# 4. More than 10 file changes detected
```

**Cost Impact**:
- Nightly (30 runs/month): $20-25/month
- Weekly (4 runs/month): **$3-4/month**
- Smart triggers (~8 runs/month): **$5-7/month**

**Savings**: Up to **$18/month = $216/year**

## Strategy 3: Combine Both (87% Cost Reduction!)

Use AWS Bedrock Provisioned + Smart Triggers for maximum savings:

| Strategy | Monthly Cost | Annual Cost | Savings |
|----------|--------------|-------------|---------|
| Anthropic API + Nightly | $25 | $300 | Baseline |
| Bedrock On-Demand + Nightly | $25 | $300 | $0 |
| Bedrock Provisioned + Nightly | $12 | $144 | $156/year |
| Anthropic API + Smart Triggers | $7 | $84 | $216/year |
| **Bedrock Provisioned + Smart** | **$3** | **$36** | **$264/year** |

**Maximum Savings**: **87% cost reduction** ($264/year saved!)

## Strategy 4: Selective Documentation Updates

Only update specific documentation types to reduce token usage.

### Skip Diagram Generation

Diagrams use more tokens. Skip if not frequently needed:

```bash
cargo run -p auto-update-docs -- \
  --use-bedrock \
  --generate-diagrams=false
```

**Savings**: ~30% token reduction

### Update High-Priority Docs Only

Modify `doc_updater.rs` to filter by priority:

```rust
// Only update high-priority docs
let updates_needed: Vec<_> = updates_needed
    .into_iter()
    .filter(|u| u.priority == UpdatePriority::High)
    .collect();
```

**Savings**: ~40-60% token reduction

## Strategy 5: Use Cheaper Models

For simple documentation updates, use cheaper models:

```bash
# Haiku is 10x cheaper than Sonnet
cargo run -p auto-update-docs -- \
  --use-bedrock \
  --model anthropic.claude-3-haiku-20240307-v1:0
```

**Claude 3 Haiku Pricing**:
- $0.25/M input tokens (vs $3 for Sonnet)
- $1.25/M output tokens (vs $15 for Sonnet)

**Savings**: ~90% cost for documentation updates!

**Recommendation**: Use Haiku for routine updates, Sonnet for major architectural changes.

## Strategy 6: Batch Mode (Future)

AWS Bedrock is adding batch mode for asynchronous processing at 50% discount.

When available:
```bash
cargo run -p auto-update-docs -- \
  --use-bedrock \
  --batch-mode \
  --max-wait-hours 24
```

**Expected Savings**: Additional 50% off Bedrock prices

## Recommended Setup

For most users, we recommend:

### Option A: Maximum Cost Efficiency
- **API**: AWS Bedrock Provisioned Throughput
- **Trigger**: Smart triggers (weekly + significant changes)
- **Model**: Claude 3 Haiku for routine updates
- **Cost**: ~**$2-3/month** (~$30/year)

### Option B: Balanced Performance
- **API**: AWS Bedrock On-Demand
- **Trigger**: Smart triggers
- **Model**: Claude 3.5 Sonnet
- **Cost**: ~**$5-7/month** (~$70/year)

### Option C: Always Up-to-Date
- **API**: AWS Bedrock On-Demand
- **Trigger**: Nightly
- **Model**: Claude 3.5 Sonnet
- **Cost**: ~**$20-25/month** (~$270/year)

## Monitoring Costs

### Track AWS Bedrock usage:

```bash
# View Bedrock costs (last 30 days)
aws ce get-cost-and-usage \
  --time-period Start=2025-01-01,End=2025-02-01 \
  --granularity MONTHLY \
  --metrics BlendedCost \
  --filter file://bedrock-filter.json

# bedrock-filter.json:
{
  "Dimensions": {
    "Key": "SERVICE",
    "Values": ["Amazon Bedrock"]
  }
}
```

### Set up cost alerts:

1. AWS Budgets: Set budget for Bedrock usage
2. CloudWatch Alarms: Alert when spend exceeds threshold
3. Cost Anomaly Detection: Automatically detect unusual costs

## Summary

| Strategy | Implementation | Savings | Effort |
|----------|----------------|---------|--------|
| Use Bedrock | `--use-bedrock` | 0-50% | Low |
| Smart Triggers | Use smart workflow | 60-75% | Low |
| Cheaper Models | `--model haiku` | 90% | Low |
| Selective Updates | Code changes | 30-60% | Medium |
| **Combined** | All of above | **85-95%** | Low-Medium |

**Best ROI**: Bedrock + Smart Triggers + Haiku model = **~$2-3/month** (save $264/year!)

## Further Reading

- [AWS Bedrock Pricing](https://aws.amazon.com/bedrock/pricing/)
- [Anthropic API Pricing](https://www.anthropic.com/pricing)
- [Claude Model Comparison](https://www.anthropic.com/claude)
