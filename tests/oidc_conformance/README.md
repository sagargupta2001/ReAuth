# OIDC Conformance Harness

This harness runs the OpenID Connect conformance suite against the ReAuth OIDC endpoints
(authorize/token/userinfo/discovery) using the basic OP test plan.

## What it does
- Clones the official conformance suite repo.
- Builds and starts the suite using Docker Compose.
- Builds the embedded UI and starts ReAuth with a CI-friendly config.
- Runs the `oidcc-basic-certification-test-plan` with discovery + static client registration.

## Run locally
```
make oidc-conformance
```

## Environment overrides
- `CONFORMANCE_SUITE_DIR` (default: `.tmp/conformance-suite`)
- `CONFORMANCE_SUITE_REF` (default: `release-v5.1.35`)
- `CONFORMANCE_BASE_URL` (default: `https://localhost:8443/`)
- `CONFORMANCE_MTLS_URL` (default: `https://localhost.emobix.co.uk:8444/`)
- `REAUTH_BASE_URL` (auto-detected if unset)
- `REAUTH_WAIT_URL` (default: `http://localhost:3000`)
- `REAUTH_REALM` (default: `master`)
- `OIDC_CLIENT_ID` (default: `reauth-conformance`)
- `ADMIN_USERNAME` / `ADMIN_PASSWORD` (default: `admin` / `admin`)

## Notes
- The harness assumes ReAuth is serving the embedded UI so the conformance suite can drive
  login and consent.
- Browser automation clicks the consent "Allow" button using `data-intent="allow"`.
