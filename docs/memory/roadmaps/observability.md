# Feature Roadmap: Logging, Auditing & Observability

## Goal
- Provide actionable logs, audit trails, and performance telemetry for debugging, security, and compliance.

## Vision (ReAuth Observability Engine)
- **Telemetry Layer (Rust):** Custom tracing layer feeding the built-in UI. Optional OTLP export later.
- **Events as Logs:** timestamp, level, message, request_id (plus structured metadata).
- **Spans as Traces:** trace_id, span_id, parent_id, name, start_time, duration.
- **Storage Sink:** dedicated SQLite DB (`reauth_telemetry.db`) optimized for high write throughput (WAL, synchronous=NORMAL).
- **Admin API:** `/api/system/observability` endpoints for logs, traces, and cache control.
- **UI:** Logs Explorer, Traces Waterfall, Cache Manager (dark-mode friendly, CloudWatch-inspired).

## Current implementation (code-aligned)
- [x] API latency logging is standardized (route, status, duration).
- [x] Correlation/request IDs are generated and propagated.
- [x] W3C `traceparent` parsing and propagation.
- [x] Request logs include request_id, trace_id, span_id, user_id, realm (when available).
- [x] JSON log output toggle (compact vs JSON) + configurable target display.
- [x] Error responses include request_id and stable error codes (JSON).
- [x] In-memory log bus + `/api/logs/ws` live stream for UI.
- [x] Telemetry persistence (logs + traces) is implemented (SQLite).
- [x] Observability admin API endpoints for logs/traces/cache (EVENT_READ gated).
- [x] Trace spans emitted for key middleware/service operations (request context + nested spans).
- [x] Baseline metrics exposed (request count + latency histogram).
- [x] Cache stats/flush support namespaces (per-namespace stats + flush).
- [x] RBAC audit events are persisted and queryable.

## MVP scope (prioritized)
1. **Audit persistence (RBAC)** (done)
   - Persist RBAC change events with actor, target, action, and timestamp.
   - Provide query API for recent audits.
2. **Telemetry storage (MVP)** (done)
   - Create `reauth_telemetry.db` with WAL + optimized pragmas.
   - Persist request logs + trace records.
3. **Observability Admin API (MVP)** (done)
   - `GET /api/system/observability/logs` with filters (`level`, `search`, `limit`).
   - `GET /api/system/observability/traces` (top-level request traces, latencies).
   - `GET /api/system/observability/traces/{trace_id}` (spans for waterfall).
   - `GET /api/system/observability/metrics` (request count + latency histogram).
   - `GET /api/system/observability/cache/stats`.
   - `POST /api/system/observability/cache/flush`.
4. **Structured logging (in progress)**
   - Standardize key fields across handler logs (request_id, user_id, realm, trace_id, span_id).
5. **UI MVP**
   - Observability layout with Logs/Traces/Cache tabs + time range selector.
   - Logs Explorer: search/filters, live tail, dense table + JSON expansion.
   - Traces view: request list + waterfall chart.
   - Cache Manager: stats, namespaces, purge actions, guarded global flush.

## Enhancements (later)
- Outbound propagation of traceparent and request_id.
- Metrics baseline (request count, latency histograms, auth failures, DB latency).
- Tracing spans across handlers/services + sampling.
- Log redaction/PII policy enforcement.
- OTLP export (OpenTelemetry) + external backends.
- Alerting and anomaly detection.

## Open questions
- What retention period is required for audit and telemetry records?
- What sampling strategy is acceptable for high-volume logs/spans?
