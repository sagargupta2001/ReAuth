# Feature Roadmap: Logging, Auditing & Observability

## Goal
- Provide actionable logs, audit trails, and performance telemetry for debugging and compliance.

## Current state (code-aligned)
- [ ] API latency logging is not standardized.
- [ ] Audit events exist in-memory but not persisted for reporting.
- [ ] No structured tracing/metrics surfaced.

## MVP scope (prioritized)
1. **API latency logging**
   - Log every API call with route, status, and response time.
   - Add correlation IDs for request tracing.
2. **Audit persistence (RBAC)**
   - Persist RBAC change events with actor, target, action, and timestamp.
   - Provide query API for recent audits.
3. **Structured logging**
   - Standardize log format (JSON) with key fields (request_id, user_id, realm_id).

## Later
- Metrics pipeline (Prometheus/OpenTelemetry).
- Distributed tracing.
- Alerting and anomaly detection.

## Open questions
- What retention period is required for audit records?
