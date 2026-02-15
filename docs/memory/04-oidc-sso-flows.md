# OIDC and SSO Flows

This doc is limited to behavior implemented in code today. It focuses on the Authorization Code + PKCE flow and the SSO cookie path.

## Endpoints (current)
- Authorize: `GET /api/realms/{realm}/oidc/authorize`
- Token: `POST /api/realms/{realm}/oidc/token` (form urlencoded)
- JWKS: `GET /api/realms/{realm}/oidc/.well-known/jwks.json`

## OIDC authorization (authorize -> login UI)
```mermaid
sequenceDiagram
  participant UA as User Agent
  participant OIDC as OIDC Authorize Handler
  participant SVC as OidcService
  participant DB as AuthSessionRepo

  UA->>OIDC: GET /oidc/authorize (client_id, redirect_uri, response_type, scope, state, nonce, code_challenge, code_challenge_method)
  OIDC->>SVC: initiate_browser_login(realm_id, OidcRequest)
  SVC->>SVC: validate client + redirect_uri
  SVC->>DB: create AuthenticationSession with context.oidc
  OIDC-->>UA: Set reauth_login_session cookie + Redirect /#/login?realm={realm}&{original_query}
```

Notes (from code):
- `initiate_browser_login` validates the client and redirect URI before creating the session.
- OIDC context is stored in `AuthenticationSession.context.oidc` for use after login.

## Login UI and code issuance
The login UI uses `AuthFlowExecutor` and the browser flow template. On success, the backend issues an authorization code and returns a redirect URL to the client.

```mermaid
sequenceDiagram
  participant UI as AuthFlowExecutor
  participant API as /auth/login handlers
  participant Exec as FlowExecutor
  participant OIDC as OidcService

  UI->>API: GET /api/realms/{realm}/auth/login (with OIDC params)
  API->>Exec: execute(session_id, None)
  Exec-->>API: Challenge | Success | Failure
  API-->>UI: JSON response + reauth_login_session cookie

  UI->>API: POST /api/realms/{realm}/auth/login/execute (credentials)
  API->>Exec: execute(session_id, input)
  Exec-->>API: Success
  API->>OIDC: create_authorization_code(...)
  API-->>UI: JSON { status: "redirect", url: redirect_uri?code=...&state=... }
  UI->>UA: browser redirect to client callback
```

Notes (from code):
- The login handler checks `AuthenticationSession.context.oidc` to decide whether to issue an auth code.
- If a valid SSO token is present in context, the handler skips creating a new root session.

## Token exchange (code -> tokens)
```mermaid
sequenceDiagram
  participant Client as OIDC Client
  participant Token as /oidc/token
  participant OIDC as OidcService
  participant Repo as OIDC Repo
  participant Auth as AuthService

  Client->>Token: POST grant_type=authorization_code, code, redirect_uri, client_id, code_verifier
  Token->>OIDC: exchange_code_for_token(code, verifier)
  OIDC->>Repo: find auth code
  OIDC->>OIDC: verify PKCE (S256)
  OIDC->>Repo: delete auth code
  OIDC->>Auth: create_session(user, client_id)
  OIDC-->>Token: token response + refresh token
  Token-->>Client: JSON token response + Set-Cookie reauth_refresh_token
```

Notes (from code):
- PKCE verification always uses SHA-256; `plain` is not supported.
- The auth code TTL is 300 seconds.
- The token response includes `access_token`, `id_token`, `token_type`, and `expires_in`.

## SSO cookie path (browser flow)
The browser flow template starts with a cookie authenticator. If a valid refresh token is present, it short-circuits to success.

```mermaid
sequenceDiagram
  participant UA as User Agent
  participant API as /auth/login
  participant Exec as FlowExecutor
  participant Cookie as CookieAuthenticator
  participant Sess as SessionRepository

  UA->>API: GET /auth/login (no prompt=login)
  API->>API: read reauth_refresh_token cookie
  API->>Exec: execute(session_id, None) with context.sso_token_id
  Exec->>Cookie: execute()
  Cookie->>Sess: find refresh token by id
  alt token valid + realm match
    Cookie-->>Exec: FlowSuccess
    Exec-->>API: Success
  else
    Cookie-->>Exec: Continue
    Exec-->>API: Challenge
  end
```

Notes (from code):
- SSO token is the `reauth_refresh_token` cookie.
- Realm isolation is enforced: token realm must match session realm.
- If `prompt=login`, the SSO cookie is ignored.
