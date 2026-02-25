# Feature Roadmap: Webhooks & Event Engine

## Goal
- Provide reliable, async delivery of domain events to HTTP webhooks and gRPC plugins without slowing the request path.

## Current state (code-aligned)
- Domain events are a fixed enum in `crates/reauth_core/src/domain/events.rs` covering user, group, and role changes.
- The event bus is in-process only: `InMemoryEventBus` uses `tokio::sync::broadcast` with capacity 1024.
- Events are published by application services like `UserService` and `RbacService`.
- Subscribers registered at startup are `CacheInvalidator` (permission cache invalidation).
- Outbox rows are written transactionally for user + RBAC writes, and a background worker dispatches them.
- HTTP webhook delivery is implemented with HMAC signing and per-target logging.
- Webhook endpoints support `http_method` (POST/PUT) and dispatchers honor it (test + replay included).
- gRPC plugin delivery is routed via the outbox worker with response handling and logging.
- Retry/backoff with jitter + dead-letter + circuit breaker are implemented.
- Webhook admin API supports CRUD, enable/disable, subscription toggles, and delivery log listing.
- Event Routing UI ships with tabs, detail inspector, inline edit/delete, refresh, and breadcrumb navigation.

## Priority plan (tracked)
- [x] P0: Add `event_outbox` + webhook tables in primary DB (migration).
- [x] P0: Add `delivery_logs` table in telemetry DB and enable WAL.
- [x] P0: Make outbox inserts transactional with domain writes (user + RBAC flows).
- [x] P0: Formalize event envelope (`event_id`, `event_type`, `event_version`, `occurred_at`, `realm_id`, `actor`, `data`).
- [x] P1: Event router that fans out to HTTP + gRPC targets.
- [x] P1: Delivery logging per target + latency + response capture.
- [x] P1: Retry/backoff + dead-letter + circuit breaker.
- [x] P1: Admin API for webhook CRUD + test delivery + enable/disable + subscription toggles.
- [x] P1: Delivery log inspection endpoints.
- [x] P1: Event Routing UI for webhooks/plugins + delivery inspector.
- [x] P1: Webhook HTTP method support (POST/PUT) end-to-end.
- [x] P1: Omni Search entries for Event Routing + DB-backed Webhook search.

## Next (implementation details)
- Retry queue: exponential backoff with jitter and a dead-letter state after max attempts.
- Split storage into `reauth_primary.db` (auth data + event_outbox) and `reauth_telemetry.db` (delivery logs; audit can move later) with WAL enabled.

## Later
- Per-realm quotas and rate limiting for webhooks and plugins.
- Event filtering rules beyond event type (realm, client_id, predicate rules).
- Payload encryption at rest for delivery logs (if stored).
- Replay tooling for backfills and incident recovery.

## Decisions (resolved)
- v1 queue backing: SQLite-only transactional outbox. Redis can be added behind the same port later.
- Delivery semantics: at-least-once with idempotency keys (`Reauth-Event-Id`).
- Payload storage: inline JSON in delivery logs; optionally zstd-compress large payloads into a BLOB column.

## Risks / dependencies (mitigations)
- SQLite write contention: segregate DBs so delivery logs do not lock auth data; keep outbox in primary for transactional guarantees.
- Event schema changes: use an envelope with explicit `event_version` and plugin-declared supported version.
- Retry storms: exponential backoff with jitter plus circuit breaker and per-target disable.

## Implementation checklist (with reasons)
- [x] Add `event_outbox` table written in the same transaction as domain changes (user + RBAC flows).
Reason: guarantees event persistence when state changes commit.
Problem solved: eliminates lost events when process crashes between write and publish.
- [x] Create `delivery_logs` table in `reauth_telemetry.db` and switch to WAL mode.
Reason: isolates high-write webhook traffic from auth reads/writes.
Problem solved: avoids login/RBAC writes blocking on delivery logging.
- [x] Build outbox worker that polls pending rows and hands off to router (HTTP + gRPC delivery logging).
Reason: decouples request latency from delivery work.
Problem solved: core API stays fast while events are delivered asynchronously.
- [x] Implement event envelope (`event_id`, `event_type`, `event_version`, `occurred_at`, `realm_id`, `actor`, `data`).
Reason: provides stable contract and versioning for plugins/webhooks.
Problem solved: backward compatibility during payload evolution.
- [x] Add `Reauth-Event-Id` header to HTTP and include in gRPC envelope.
Reason: enables consumer-side deduplication.
Problem solved: safe at-least-once delivery without exactly-once complexity.
- [x] Add HTTP signing (`Reauth-Signature`) and per-endpoint secret rotation.
Reason: prevents spoofed webhook calls.
Problem solved: integrity and authenticity of outbound events.
- [x] Implement backoff schedule with jitter (1m, 5m, 30m, 2h, 12h) and max attempts.
Reason: smooths retries and avoids synchronized bursts.
Problem solved: retry storms that overwhelm targets.
- [x] Add circuit breaker state on endpoints (disable after N consecutive failures).
Reason: prevents endlessly hammering dead endpoints.
Problem solved: self-amplifying failure loops and upstream outages.
- [x] Extend plugin manifest with `supported_event_version` and add version mapping in router.
Reason: legacy plugins can still operate when event schema evolves.
Problem solved: plugin crashes due to breaking payload changes.
- [x] Add webhook HTTP method support (POST/PUT) and surface it in the UI table.
Reason: aligns with downstream expectations for signature verification and routing.
Problem solved: endpoints that require PUT can be supported without custom proxies.
- [ ] Store payload inline; use zstd compression for large payloads.
Reason: IAM events are small; compression keeps DB size predictable.
Problem solved: avoids premature blob-store complexity while keeping storage efficient.
- [x] Add DB-backed Webhook search results to Omni Search.
Reason: quick access to specific endpoints during incident response.
Problem solved: removes manual scanning of long webhook lists in large realms.
