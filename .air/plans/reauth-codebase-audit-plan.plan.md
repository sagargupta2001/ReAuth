# Codebase Audit & Refactoring Plan: ReAuth Project

This document outlines a comprehensive audit and refactoring strategy to elevate the ReAuth project (Rust backend + React UI) to senior-engineer-level standards. The plan ensures alignment with SOLID principles, FSD (Feature-Sliced Design), and Clean Architecture, while addressing existing technical debt.

---

## A. Audit Report

### Critical Severity
1. **Clean Architecture Violations (Backend):**
   - `src/application/delivery_replay_service.rs` and `src/application/webhook_service.rs` execute raw `sqlx::query(...)` directly in the application layer. This breaks the `ports/adapters` boundary; the application layer is leaking database concerns and tightly coupling with SQLite.
2. **FSD Violations (Frontend):**
   - `ui/src/shared/api/client.ts` imports directly from `@/entities/session/model/sessionStore`. The `shared` layer is strictly foundational and must never depend on higher layers (`entities`, `features`, etc.).

### High Severity
1. **"God Files" & SRP Violations (Backend):**
   - Multiple backend files exceed 1000 lines and mix multiple responsibilities. Examples include `rbac_service/mod.rs` (1356 lines), `harbor/service.rs` (1865 lines), `theme_service.rs` (1717 lines), and web handlers like `rbac_handler.rs` (1390 lines) and `theme_handler.rs`.
2. **God Components (Frontend):**
   - `FluidInspector.tsx` (1490 lines), `OmniCommandPalette.tsx` (1155 lines), and `FluidLoginScreen.tsx` (1039 lines). These components mix state management, styling, sub-component rendering, and utility functions (e.g., `parseColor`).
3. **Domain Layer Leak (Backend):**
   - `src/domain/compiler/validator.rs` imports `crate::application::runtime_registry::RuntimeRegistry`. The `domain` layer must be pure and entirely unaware of the `application` layer.

### Medium Severity
1. **Handler Orchestration (Backend):**
   - Handlers like `src/adapters/web/auth_handler.rs` contain complex business logic (verifying capabilities, managing SSO root cookies, determining session state) instead of delegating entirely to application services (`AuthService` / `FlowService`).
2. **FSD Violations (Frontend Contexts):**
   - Multiple `features` and `widgets` import `ThemeContext` from `@/app/providers/ThemeContext`. Features and widgets should not reach upwards into the `app` layer.
   - `ui/src/entities/flow/config/nodeTypes.ts` imports directly from `@/features/flow-builder/...`. Entities cannot depend on features.

### Low Severity
1. **Unused Dependencies:**
   - `boa_engine` (JavaScript engine) is present in `Cargo.toml` but is unused anywhere in the codebase.

---

## B. Refactoring Plan

### Phase 1: Quick Wins (Low Effort, High Impact)
- **Dependency Cleanup:**
  - **Problem:** `boa_engine` inflates the binary size and compilation time unnecessarily.
  - **Solution:** Remove `boa_engine` from `Cargo.toml`.
  - **Impact:** Faster builds, smaller binary.
- **FSD Inversion Fixes (UI):**
  - **Problem:** `ThemeContext` lives in `app/`, and `shared/api/client.ts` imports `entities`.
  - **Solution:** Move `ThemeContext` to `shared/theme/ThemeContext.tsx`. Refactor `client.ts` to accept an injected interceptor (e.g., `injectAuthInterceptor(getToken, onRefresh)`) during app bootstrap, severing the dependency on `entities`.
  - **Impact:** Restores strict FSD hierarchy.
- **Domain Decoupling (Rust):**
  - **Problem:** `domain` depends on `application`.
  - **Solution:** Define a `NodeRegistry` trait in `src/domain/flow/` that `GraphValidator` relies on. Let `application::runtime_registry::RuntimeRegistry` implement this trait.
  - **Impact:** Clean architecture restored for the core domain.

### Phase 2: Medium-Level Refactors
- **Abstract SQL from Application Services:**
  - **Problem:** `delivery_replay_service.rs` and `webhook_service.rs` use `sqlx`.
  - **Solution:** Extract these database interactions into `TelemetryRepository` and `WebhookRepository` trait boundaries within `src/ports/`. Implement the logic in `src/adapters/persistence/`.
  - **Impact:** True Hexagonal architecture, better testability.
- **Break Down UI God Components:**
  - **Problem:** `FluidInspector.tsx` and others are unmaintainable.
  - **Solution:** Extract pure utilities (e.g., `parseColor`, `normalizeColorValue`) to `ui/src/shared/lib/colorUtils.ts`. Split the 1400-line inspector into smaller compositional components: `<ColorPicker />`, `<TypographyControls />`, `<SpacingControls />`.
  - **Impact:** Vastly improved readability and component reusability.

### Phase 3: Larger Architectural Improvements
- **Deconstruct Backend God Files:**
  - **Problem:** `rbac_service` and `harbor/service.rs` violate SRP.
  - **Solution:** Submodule the services. Break `rbac_service/mod.rs` into a facade that delegates to `roles.rs`, `groups.rs`, and `assignments.rs`. Do the same for massive handlers like `rbac_handler.rs`.
  - **Impact:** Scalable, modular codebase matching senior-engineer conventions.
- **Extract Business Logic from Web Handlers:**
  - **Problem:** `auth_handler.rs` handles complex state resolutions.
  - **Solution:** Push all cookie determination, realm capability checks, and session resolution into `src/application/auth_service`. The handler should only parse the request, invoke `auth_service.start_public_flow(...)`, and map the result to an HTTP response.
  - **Impact:** Centralized business logic, isolated infrastructure parsing.

---

## C. Proposed Improved Structure

### Backend (Rust)
```text
src/
├── application/
│   ├── rbac_service/
│   │   ├── mod.rs (Facade/Orchestrator)
│   │   ├── roles.rs (Role-specific logic)
│   │   ├── groups.rs (Group-specific logic)
│   │   └── assignments.rs
├── adapters/
│   ├── web/
│   │   ├── rbac/
│   │   │   ├── mod.rs (Router setup)
│   │   │   ├── role_handlers.rs
│   │   │   └── group_handlers.rs
├── domain/
│   ├── flow/
│   │   └── node_registry.rs (Trait definition for Domain purity)
```

### Frontend (UI)
```text
ui/src/
├── shared/
│   ├── api/
│   │   ├── client.ts
│   │   └── interceptors.ts (Pure abstraction, initialized at app root)
│   ├── theme/
│   │   └── ThemeContext.tsx (Moved down from app/)
├── features/
│   ├── fluid/
│   │   ├── components/
│   │   │   ├── inspector/
│   │   │   │   ├── FluidInspector.tsx (Composition root)
│   │   │   │   ├── ColorPicker.tsx
│   │   │   │   └── TypographyControls.tsx
```

---

## D. Specific Recommendations

1. **Dependency Injection in React (`shared/api/client.ts`):**
   Instead of:
   ```typescript
   import { useSessionStore } from '@/entities/session/model/sessionStore'
   ```
   Do:
   ```typescript
   // shared/api/client.ts
   let refreshCallback: () => Promise<string>;
   export function setRefreshCallback(cb: () => Promise<string>) { refreshCallback = cb; }
   ```
   Then in `app/providers/index.tsx`, call `setRefreshCallback(...)` with the store logic.

2. **SOLID - Dependency Inversion in Rust:**
   The `DeliveryReplayService` should not construct a `reqwest::Client` directly. Create an `HttpDeliveryClient` port in `src/ports/` to handle the actual outgoing POST request and signature signing. This allows mocking the network layer completely during unit tests.

3. **React Performance & Rendering:**
   In massive files like `FluidLoginScreen.tsx`, heavy use of local component state causes massive re-renders. By splitting the UI into smaller components and pushing state down (or using Zustand selectors more granularly), you drastically reduce React reconciliation times.