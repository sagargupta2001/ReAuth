# Feature Roadmap: Webhooks & Event Engine

## Goal
- Provide reliable, async delivery of domain events to HTTP webhooks and gRPC plugins without slowing the request path.

## Current state (code-aligned)
- Domain events are a fixed enum in `crates/reauth_core/src/domain/events.rs` covering user, group, and role changes.
- The event bus is in-process only: `InMemoryEventBus` uses `tokio::sync::broadcast` with capacity 1024.
- Events are published by application services like `UserService` and `RbacService`.
- Subscribers registered at startup are `CacheInvalidator` (permission cache invalidation) and `PluginEventGateway` (gRPC delivery).
- Plugin delivery is fire-and-forget gRPC with JSON payloads and no response handling, retries, or persistence.
- There is no HTTP webhook delivery, no delivery logs, and no retry or dead-letter mechanism.

## Now (design + groundwork)
- Formalize an event envelope: `event_id`, `event_type`, `event_version`, `occurred_at`, `realm_id`, and `actor` metadata.
- Define storage tables for webhook configuration and delivery tracking: `webhook_endpoints`, `webhook_subscriptions`, `event_outbox`, `delivery_logs`.
- Add a delivery state model (status, attempt_count, last_error, next_attempt_at) to enable retry scheduling.
- Decide on queue backing: SQLite outbox + worker loop by default, optional Redis later.
- Add configuration knobs: concurrency limits, per-target timeouts, retry/backoff policy, max payload size.

## Next (implementation)
- Implement the unified Event Router worker that consumes the outbox and fans out to HTTP and gRPC targets.
- HTTP dispatcher: HMAC-SHA256 signing with per-endpoint secret, `Reauth-Signature` header, idempotency key header.
- gRPC dispatcher: reuse manifest subscription filtering and standardize a protobuf envelope.
- Delivery logger: write one delivery record per attempt and mark success/failure with latency.
- Retry queue: exponential backoff with jitter and a dead-letter state after max attempts.
- Admin APIs for webhooks: CRUD endpoints, test delivery, pause/resume, and delivery log inspection.

## Later
- UI for webhook management and delivery troubleshooting.
- Per-realm quotas and rate limiting for webhooks and plugins.
- Event filtering rules beyond event type (realm, client_id, predicate rules).
- Payload encryption at rest for delivery logs (if stored).
- Replay tooling for backfills and incident recovery.

## Risks / dependencies
- SQLite concurrency and long-running delivery workers could contend with primary DB writes.
- Event schema changes require versioning and backwards compatibility for plugins.
- Retry storms can overload downstream systems without per-target rate limits.

## Open questions
- Should the first release be SQLite-only outbox, or include optional Redis from day one?
- Is at-least-once delivery sufficient if we provide idempotency keys, or do we need exactly-once?
- Where should large payloads live: inline in delivery logs or in a separate blob store?
