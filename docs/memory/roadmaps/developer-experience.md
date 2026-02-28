# Feature Roadmap: Developer Experience (SDKs)

## Goal
- Make ReAuth integration take under 5 minutes with first-class SDKs.

## Current state
- No official SDKs.

## Now
- React SDK (`@reauth/react`) with `AuthProvider` for session state and silent refresh.
- React hooks `useAuth()` and `useUser()` with global loading and error handling.
- Node.js/Express SDK (`@reauth/node`) with JWT verification middleware and JWKS caching.
- Node helpers for token validation in service-to-service flows.

## Next
- Pre-built React components: `SignIn`, `SignUp`, `UserProfile`.
- Typed API client generation and error normalization.
- First-party examples and templates.

## Later
- Next.js and Remix adapters.
- Mobile SDKs (React Native, Flutter).

## Risks / dependencies
- Requires stable API contracts and error semantics.
- JWT validation must handle rotation and kid changes safely.

## Open questions
- Client-side refresh strategy (iframe vs service worker).
- Default UI design system vs user-supplied styling.
