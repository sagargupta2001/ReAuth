# Feature Roadmap: Must-Have Identity Flows

## Goal
- Provide standard user journeys built on the flow engine: registration, verification, recovery, and MFA.

## Current state (code-aligned)
- Default flows exist (browser, registration, reset) but lack verification and MFA steps.
- No password policy enforcement beyond basic validation.
- No email verification or recovery token workflows.
- Flow engine now supports async pause/resume via action tokens + waiting UI.

## Now
- Registration flow with password policy validation and zxcvbn scoring.
- Email verification step using OTP or magic link.
- Flow pause/resume integration for verification.
- Credential recovery flow with secure token generation and validation.
- Invalidate all sessions on password reset success.

## Next
- MFA Phase 1: TOTP enrollment and enforcement based on realm or client policy.
- Recovery safeguards: rate limits, token reuse detection, and audit events.

## Later
- WebAuthn and backup codes.
- Step-up authentication for sensitive actions.

## Risks / dependencies
- Requires reliable email delivery and templating strategy.
- Session invalidation must be consistent across token families.

## Open questions
- Password policy defaults and per-realm overrides.
- OTP vs magic link priority for verification.
- MFA policy scoping by realm vs client.
