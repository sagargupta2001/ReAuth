# Identity Brokering Operations

Purpose: rollout, validate, and operate inbound OAuth/OIDC identity brokering in production realms.

## Scope

- Covers the current inbound identity-brokering surface implemented from `docs/specs/oauth-inbound-identity-brokering.md`.
- Focuses on realm-scoped external providers such as Google, GitHub, Microsoft, Apple, generic OIDC, and generic OAuth2.
- Does not cover LDAP / Active Directory or SCIM follow-up work.

## Operator Prerequisites

- Set a correct public base URL in `server.public_url`.
- Serve the public auth surface over HTTPS in the target environment.
- Configure a stable secret-encryption key so provider secrets are not re-encrypted with a fallback key.
- Ensure callback routes are reachable at:
  - `/api/realms/{realm}/auth/oauth/{alias}/start`
  - `/api/realms/{realm}/auth/oauth/{alias}/callback`
- Confirm observability is available so `category=idp` events can be queried after rollout.

## Rollout Runbook

1. Verify system prerequisites.
   - Confirm `server.public_url` matches the real external origin.
   - Confirm the realm login page is reachable through the same public origin used in provider callback registration.

2. Enable brokering at the realm level.
   - Open `Realm Settings -> General -> Identity Brokering`.
   - Enable the brokering master switch.
   - Set realm defaults for JIT, email auto-link, and unlink protection before adding providers.

3. Configure the broker start throttle.
   - Open `Realm Settings -> Identity Brokering`.
   - Review the default throttle of `30` starts per IP per provider per `10` minutes.
   - Tighten it for internet-facing realms if abuse volume is expected.

4. Add a provider from presets first.
   - Prefer a preset over manual entry when available.
   - Supply client ID and client secret.
   - Review endpoints, scopes, PKCE, claim mapping, login/link/JIT flags, and branding.

5. Run `Test Connection`.
   - Confirm discovery, token endpoint, userinfo, and JWKS checks are green where applicable.
   - If discovery or JWKS refreshes occurred, verify cache timestamps updated on the provider detail page.

6. Validate flow exposure.
   - Confirm the provider is enabled and `allow_login=true`.
   - Confirm the realm flow actually exposes the provider through login buttons, `collect_idp_choice`, or a direct `oauth_idp` branch.

7. Execute an end-to-end sign-in in a private browser session.
   - Validate the redirect, upstream consent, callback, and ReAuth landing page.
   - Confirm the expected outcome branch:
     - existing link
     - verified email auto-link
     - manual link
     - JIT provisioning
     - failure/conflict

8. Check observability immediately after the test.
   - Query Logs with `category=idp`.
   - Expect the ordered flow:
     - `idp_redirect_issued`
     - `idp_callback_received`
     - callback success/failure family
     - `idp_user_linked` or `idp_jit_provisioned` when applicable

9. Roll out additional providers one at a time.
   - Keep each provider’s scopes, JIT policy, and email-link policy documented in the realm change log.
   - Do not batch multiple new providers into one unverified rollout.

## Validation Checklist

- Provider button appears only where the active flow exposes it.
- Provider order, label, icon, and color match admin configuration.
- Callback returns to the correct realm and preserves the auth session.
- `category=idp` logs are queryable and include provider alias plus auth session correlation.
- User credentials show the federated identity after successful link or JIT.
- Unlink protection behaves as intended for the realm policy.
- Provider activity page shows recent broker events and failure counts.

## Rollback

Use the least-destructive option first.

1. Disable the provider.
   - Prevents new logins and links through that provider.
   - Preserves linked federation rows for later recovery.

2. Disable realm brokering.
   - Removes provider exposure from the login experience for the whole realm.
   - Useful when a shared upstream issue affects multiple providers.

3. Revert the realm flow.
   - Publish the prior flow version if the issue is flow-composition specific.

4. Hard delete the provider only when necessary.
   - This also removes linked federation rows if explicit hard delete is chosen.
   - Prefer soft delete for incident response.

## Incident Triage

- `idp_start_rate_limited`
  - Indicates abuse or aggressive automation on the `/start` endpoint.
  - Review the realm IdP throttle before widening access.

- `idp_callback_invalid_request`
  - Usually missing `code` or `state`.
  - Check reverse proxy behavior, callback URL registration, and user/browser interruption patterns.

- `idp_callback_upstream_error`
  - The upstream provider returned an explicit error.
  - Check provider app policy, consent screen status, and account-level restrictions.

- `idp_callback_session_mismatch`
  - The upstream callback succeeded but the browser no longer had the matching auth session cookie.
  - Check cookie domain/path behavior, SameSite issues, proxy origin mismatches, and user multi-tab behavior.

- `idp_state_mismatch`
  - Replay, expiry, or concurrent callback consumption.
  - Check for duplicate callback delivery or very slow upstream completion.

- `idp_pkce_failure`
  - Lost or mismatched verifier.
  - Treat as suspicious unless explained by cookie/session breakage.

- `idp_token_exchange_failure`
  - Upstream token endpoint failure or malformed response.
  - Check secret rotation, redirect URI registration, and upstream availability.

- `idp_userinfo_failure`
  - Upstream profile fetch failure or provider-specific fetch failure such as GitHub emails lookup.
  - Check scopes first, then endpoint behavior.

## Provider Notes

### Google

- Use the OIDC preset when possible.
- Expect discovery and JWKS to populate automatically.
- Keep `openid email profile` in scopes unless there is a specific reason to narrow them.
- Verified email is commonly available and is a good fit for verified email auto-link if the realm wants that behavior.

### GitHub

- Uses OAuth2 semantics, not OIDC.
- Primary email may require the secondary `/user/emails` fetch path.
- Keep `read:user` and `user:email` scopes so canonical email resolution works.
- Expect `email_verified` to come from the emails endpoint, not the base profile alone.

### Microsoft

- Prefer the OIDC preset and verify the tenant-specific issuer choice during setup.
- Confirm the correct app registration audience and tenant routing before rollout.
- Review whether the tenant’s email claims are reliable enough for auto-link in your environment.

### Apple

- Treat Apple as higher-friction operationally.
- Email may not be stable or repeatedly returned the same way after the first consent depending on app configuration.
- Use conservative auto-link/JIT settings unless the operator has already validated Apple-specific behavior for the target app.

### Generic OIDC

- Prefer discovery-driven setup first.
- Verify issuer, authorization endpoint, token endpoint, userinfo endpoint, and JWKS URI after discovery refresh.
- Confirm the upstream provider returns `sub`, and ideally `email` plus `email_verified`.

### Generic OAuth2

- Requires manual endpoint correctness because discovery is not available.
- Confirm userinfo shape early because claim mapping and subject extraction depend on it.
- Avoid enabling email auto-link unless the upstream profile has a trustworthy verified-email signal.

## Safe Defaults

- Keep PKCE enabled.
- Keep start throttling enabled.
- Prefer manual linking over automatic email linking unless verified-email behavior is well understood.
- Start with JIT disabled for enterprise or ambiguous providers, then enable it intentionally.
- Prefer soft delete over hard delete when decommissioning a provider.
