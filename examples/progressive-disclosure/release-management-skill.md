---
name: release-management
description: |
  Comprehensive release management workflow covering version control, build orchestration,
  testing, deployment, monitoring, and rollback procedures across multiple environments.
  Use when preparing production releases, managing deployments, or troubleshooting release failures.
argument-hint: "[prepare|build|test|deploy|monitor|rollback] [--environment staging|production] [--version x.y.z]"
allowed-tools: Bash, Read, Write, Grep, Glob
user-invocable: true
disable-model-invocation: false
---

# Release Management Skill

Orchestrates the complete release lifecycle from version tagging through production deployment and post-release monitoring.

## Core Workflow

The release process follows these high-level stages:

1. **Prepare** — Version bumping, changelog generation, pre-flight checks
2. **Build** — Compilation, asset bundling, artifact generation
3. **Test** — Integration tests, smoke tests, security scans
4. **Deploy** — Staged rollout with health checks
5. **Monitor** — Metrics tracking, error monitoring, performance analysis
6. **Rollback** — Emergency procedures if issues detected

Each stage has detailed procedures, edge cases, and diagnostic guidance extracted to reference files for context economy.

## Invocation Modes

### `/release-management prepare --version X.Y.Z`

Prepares a new release:
1. Validates semantic versioning format
2. Updates version in package files (package.json, Cargo.toml, pyproject.toml)
3. Generates changelog from git commits since last tag
4. Runs pre-flight validation (tests pass, no uncommitted changes, branch is main)
5. Creates git tag and pushes to remote

**Pre-flight checks:**
- Working directory clean (no uncommitted changes)
- Currently on main/master branch
- All CI checks passing on HEAD
- Version doesn't already exist as tag

### `/release-management build --environment [staging|production]`

Builds release artifacts:
1. Cleans previous build artifacts
2. Installs dependencies (lockfile-only, no updates)
3. Runs build command based on detected project type
4. Generates build manifest (artifact hashes, dependency versions)
5. Uploads artifacts to staging or production artifact registry

**Supported project types:**
- Node.js: npm/yarn build
- Rust: cargo build --release
- Python: poetry build or python setup.py sdist
- Go: go build with version injection

### `/release-management test --environment staging`

Runs test suite against deployed staging environment:
1. Smoke tests (basic endpoint health)
2. Integration tests (cross-service communication)
3. Security scans (dependency vulnerabilities, SAST)
4. Performance benchmarks (latency, throughput)
5. Generates test report with pass/fail summary

**Test failure handling:**
If any test fails, the release is blocked. See failure diagnostics reference for remediation procedures.

### `/release-management deploy --environment [staging|production] --strategy [rolling|blue-green|canary]`

Deploys to target environment:
1. Validates deployment prerequisites (artifacts exist, tests passed, approvals obtained)
2. Executes deployment strategy
3. Runs health checks after each deployment increment
4. Monitors error rates and key metrics
5. Auto-rollback if error threshold exceeded

**Deployment strategies:**
- **Rolling**: Incremental instance replacement (5 instances at a time)
- **Blue-Green**: Parallel environment swap with instant cutover
- **Canary**: Gradual traffic shift (5% → 25% → 50% → 100%)

### `/release-management monitor --duration [5m|30m|1h|24h]`

Post-deployment monitoring:
1. Tracks error rates across services
2. Monitors latency percentiles (p50, p95, p99)
3. Checks throughput vs baseline
4. Watches custom business metrics
5. Sends alerts if anomalies detected

**Alert thresholds:**
- Error rate >1% for 5 minutes → WARNING
- Error rate >5% for 2 minutes → CRITICAL (auto-rollback if configured)
- p99 latency >2x baseline → WARNING
- Throughput <50% baseline → CRITICAL

### `/release-management rollback --to-version X.Y.Z --reason "description"`

Emergency rollback to previous version:
1. Validates target version exists and was previously deployed
2. Marks current version as "rolled back" in release registry
3. Executes fast-path deployment of target version (skips tests, uses artifacts from original deployment)
4. Monitors rollback success (error rates return to baseline)
5. Creates incident report with timeline and rollback reason

**Rollback triggers:**
- Manual operator decision
- Automated error rate threshold breach
- Critical security vulnerability discovered
- Data corruption detected

## Environment Configuration

Environments are configured in `.release-config.yaml`:

```yaml
environments:
  staging:
    artifact_registry: s3://my-app-artifacts/staging
    deployment_target: k8s-cluster-staging
    test_endpoint: https://staging.example.com
    auto_rollback_threshold: 0.05
    required_approvals: 1

  production:
    artifact_registry: s3://my-app-artifacts/production
    deployment_target: k8s-cluster-prod
    test_endpoint: https://api.example.com
    auto_rollback_threshold: 0.01
    required_approvals: 2

project:
  type: nodejs  # nodejs, rust, python, go
  build_command: npm run build
  test_command: npm test
  version_files:
    - package.json
    - package-lock.json
```

## Version Management

Follows semantic versioning (semver):
- **Major (X.0.0)**: Breaking changes, incompatible API changes
- **Minor (x.Y.0)**: New features, backwards-compatible
- **Patch (x.y.Z)**: Bug fixes, backwards-compatible

**Version bumping rules:**
- Pre-release: append `-alpha.N`, `-beta.N`, `-rc.N`
- Build metadata: append `+build.N` or `+sha.abc123`
- Version must not already exist as git tag
- Changelog must document changes since last version

## Branching Strategy

**Main branch protection:**
- All releases cut from `main` branch
- No direct commits to main (require PR)
- PR must pass CI before merge
- Release tags always point to main branch commits

**Hotfix branches:**
- Branch from production tag: `hotfix/v1.2.3-issue-description`
- Make minimal fix
- Tag as patch version
- Merge back to main

**Feature branches:**
- Branch from main: `feature/description`
- Develop feature
- Merge back to main via PR
- Version bump on next release

## Artifact Management

**Build artifacts include:**
- Compiled binaries or bundled code
- Static assets (images, CSS, JS)
- Configuration templates
- Database migration scripts
- Build manifest (SHA256 hashes, dependency versions, build timestamp)

**Artifact lifecycle:**
- Artifacts stored in environment-specific registries
- Retention: staging 7 days, production 90 days
- Artifacts immutable after upload (no overwrites)
- Artifact URLs include version for cache busting

**Artifact verification:**
- SHA256 hash checked on download
- Signature verification if GPG signing enabled
- Size sanity check (reject if >2x previous version size without explanation)

## CI/CD Integration

**Pipeline stages:**
1. **PR checks** (on every commit to feature branch):
   - Linting, formatting
   - Unit tests
   - Basic build smoke test

2. **Main branch checks** (on merge to main):
   - Full test suite
   - Security scans
   - Build all artifacts
   - Deploy to staging automatically

3. **Release pipeline** (on git tag push):
   - Build production artifacts
   - Run production test suite
   - Require manual approval
   - Deploy to production

**Pipeline failure handling:**
See reference files for detailed diagnostic procedures for common pipeline failures.

## Deployment Health Checks

**Pre-deployment checks:**
- Artifact exists and passes integrity check
- Target environment has capacity
- No ongoing incidents in target environment
- Required approvals obtained
- Maintenance window if configured

**During deployment:**
- Instance health checks (HTTP 200 on /health endpoint)
- Service mesh connectivity
- Database connectivity
- External API availability

**Post-deployment checks:**
- Error rate within threshold
- Latency within baseline
- Throughput within baseline
- Key business metrics stable

**Check failures:**
If health check fails during deployment, the deployment pauses and waits for manual intervention or auto-rollback (if configured).

## Monitoring Integration

**Metrics tracked:**
- Request rate (requests/second)
- Error rate (errors/total requests)
- Latency distribution (p50, p95, p99)
- Throughput (data transferred/second)
- Custom business metrics (signups, transactions, etc.)

**Metric sources:**
- Application logs (structured JSON)
- APM system (Datadog, New Relic, AppDynamics)
- Service mesh (Istio, Linkerd)
- Database query logs
- External API response times

**Alerting:**
- Slack/email notifications
- PagerDuty integration for critical issues
- Auto-rollback trigger on critical threshold
- Incident tracking (create ticket automatically)

## Security Considerations

**Vulnerability scanning:**
- Dependency vulnerability check (npm audit, cargo audit)
- Container image scanning (Trivy, Grype)
- SAST (static analysis) on code
- Secret detection (no API keys, passwords in code)

**Access control:**
- Production deploys require 2 approvals
- Staging deploys require 1 approval
- Rollbacks always require approval (no auto-rollback without review)
- Audit log of all release operations

**Secrets management:**
- Secrets stored in vault (HashiCorp Vault, AWS Secrets Manager)
- Secrets injected at runtime, never in artifacts
- Secrets rotated on schedule
- Secrets never logged or exposed in errors

## Failure Diagnostics

Common failure modes and diagnostics:

### Build Failures
Build failures during artifact compilation. See references/build-failure-diagnosis.md for detailed troubleshooting:
- Dependency resolution failures
- Compilation errors (type errors, missing symbols)
- Test failures blocking build
- Out of memory during build
- Disk space issues

### Test Failures
Test failures in staging or production test suite. See references/test-failure-diagnosis.md:
- Flaky tests (intermittent failures)
- Environment-specific failures (works locally, fails in CI)
- Integration test failures (service dependencies)
- Performance test failures (timeout, latency)
- Security scan failures (new CVEs)

### Deployment Failures
Deployment failures during rollout. See references/deployment-failure-diagnosis.md:
- Health check failures (new instances failing /health)
- Capacity issues (not enough resources)
- Configuration errors (wrong environment variables)
- Database migration failures
- Network connectivity issues

### Post-Deployment Issues
Issues discovered after deployment completes. See references/post-deployment-diagnosis.md:
- Error rate spike
- Latency degradation
- Throughput drop
- Memory leaks
- Database connection exhaustion

## Rollback Procedures

### Automatic Rollback
Triggered when error rate exceeds threshold during deployment:
1. Stop deployment immediately
2. Mark deployment as failed
3. Initiate rollback to previous version
4. Monitor rollback success
5. Create incident report

### Manual Rollback
Operator-initiated rollback:
1. Identify target version to roll back to
2. Use `/release-management rollback --to-version X.Y.Z --reason "..."`
3. Follow rollback health checks
4. Verify metrics return to baseline
5. Investigate root cause offline

**Rollback speed:**
- Fast-path rollback: 2-5 minutes (reuse previous artifacts)
- Full rollback: 10-15 minutes (rebuild artifacts if previous not available)

## Release Approval Workflow

**Staging releases:**
- Automatic on merge to main
- No manual approval required
- Auto-rollback on failure

**Production releases:**
- Requires 2 approvals from authorized users
- Approval can be manual (web UI) or automated (CI passes + security scan passes)
- Approval must be within 24 hours of build (stale builds rejected)
- Approval includes sign-off on changelog and version bump rationale

**Approval bypass:**
- Critical security hotfix can override 2-approval requirement
- Requires VP Engineering approval
- Must document bypass reason in release notes

## Audit Logging

All release operations logged:
- Operation type (prepare, build, deploy, rollback)
- Operator username and timestamp
- Environment (staging, production)
- Version
- Result (success, failure, timeout)
- Detailed logs stored in audit log system

**Retention:**
- Audit logs retained for 2 years
- Available for compliance audits
- Searchable by version, operator, environment

## Emergency Procedures

### Critical Production Issue
1. Assess severity (P0/P1/P2)
2. If P0: Immediate rollback to last known good version
3. If P1: Evaluate rollback vs forward fix
4. If P2: Forward fix in next release

### Database Corruption
1. Stop all deployments immediately
2. Isolate affected environment
3. Restore from backup
4. Replay transaction log
5. Verify data integrity before resuming

### Security Breach
1. Rotate all secrets immediately
2. Rollback to unaffected version
3. Audit logs for compromise timeline
4. Patch vulnerability
5. Deploy hotfix with accelerated approval

## Appendix: Command Reference

Full command syntax for all operations:

```bash
# Prepare release
release-management prepare --version 1.2.3 [--skip-tests] [--no-push]

# Build artifacts
release-management build --environment [staging|production] [--clean]

# Run tests
release-management test --environment staging [--suite integration|smoke|security|performance]

# Deploy
release-management deploy --environment production --strategy rolling [--auto-rollback]

# Monitor
release-management monitor --duration 30m [--metrics error-rate,latency,throughput]

# Rollback
release-management rollback --to-version 1.2.2 --reason "Critical bug in payment flow"

# Status
release-management status [--version 1.2.3]
```

## Appendix: Troubleshooting Checklist

Quick troubleshooting steps for common issues:

**Deployment stuck:**
1. Check health check logs
2. Verify network connectivity
3. Check resource capacity
4. Review recent configuration changes

**High error rate post-deploy:**
1. Check application logs for new errors
2. Compare error signatures to previous version
3. Check recent code changes
4. Consider immediate rollback if >5% error rate

**Performance degradation:**
1. Check database query performance
2. Review new code for N+1 queries or infinite loops
3. Check external API latency
4. Verify cache hit rates

**Failed rollback:**
1. Verify target version artifacts exist
2. Check rollback health checks
3. Manual instance-by-instance rollback if automated fails
4. Contact on-call SRE if manual rollback also fails

## Detailed Reference Material

The following sections contain detailed implementation procedures, diagnostic runbooks, and edge case handling. They are extracted to reference files for progressive disclosure:

- **Build Diagnostics**: Detailed troubleshooting for build failures (compiler errors, dependency issues, test failures)
- **Test Diagnostics**: Test failure analysis and remediation (flaky tests, environment issues, integration failures)
- **Deployment Diagnostics**: Deployment failure diagnosis (health checks, capacity, configuration, migrations)
- **Post-Deployment Diagnostics**: Production issue diagnosis (error spikes, latency, throughput, memory leaks)
- **Rollback Procedures**: Detailed rollback execution steps (automatic, manual, emergency, database)
- **Security Procedures**: Security incident response (vulnerability disclosure, secret rotation, breach response)
- **Monitoring Setup**: Monitoring configuration and metric collection (APM setup, alerting, dashboards)
- **CI/CD Configuration**: Pipeline setup and troubleshooting (GitHub Actions, GitLab CI, Jenkins)

Each reference file is loaded on-demand when the corresponding failure condition is encountered or when the user explicitly requests detailed guidance for a specific area.

---

**Skill Statistics:**
- Total lines: ~650 (before progressive disclosure extraction)
- Core workflow sections: 15
- Diagnostic procedures: 8 major categories
- Command variants: 20+ with flags
- Environment configurations: 2 (staging, production)
- Deployment strategies: 3 (rolling, blue-green, canary)
- Failure modes documented: 25+

This skill demonstrates the value of progressive disclosure: the core workflow (lines 1-300) provides sufficient guidance for successful releases, while detailed diagnostic procedures (lines 300-650) are extracted to references and loaded only when failures occur.
