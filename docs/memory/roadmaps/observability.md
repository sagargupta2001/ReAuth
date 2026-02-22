# Feature Roadmap: Logging, Auditing & Observability

## Goal
- Provide actionable logs, audit trails, and performance telemetry for debugging and compliance.

## Current state (code-aligned)
- [x] API latency logging is standardized.
- [x] Correlation/request IDs are generated and propagated.
- [x] API logs include route, status, duration, and request_id.
- [x] W3C `traceparent` parsing and propagation.
- [x] JSON log output toggle (compact vs JSON).
- [x] Error responses include request_id for JSON errors.
- [ ] Audit events exist in-memory but not persisted for reporting.
- [ ] No structured tracing/metrics surfaced.

## MVP scope (prioritized)
1. **API latency logging** (done)
   - Log every API call with route, status, and response time.
   - Add correlation IDs for request tracing.
   - Echo request IDs in responses for client correlation.
2. **Audit persistence (RBAC)** (next)
   - Persist RBAC change events with actor, target, action, and timestamp.
   - Provide query API for recent audits.
3. **Structured logging** (next)
   - Standardize log format (JSON) with key fields (request_id, user_id, realm_id).

## Later
- Metrics pipeline (Prometheus/OpenTelemetry).
- Distributed tracing.
- Alerting and anomaly detection.

## Open questions
- What retention period is required for audit records?
