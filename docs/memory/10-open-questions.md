# Open Questions

- Should `oidc_clients.client_id` be unique per realm instead of globally unique?
- Should PKCE `code_challenge_method=plain` be supported or explicitly rejected?
- Should `auth_sessions.execution_state` and `last_ui_output` be used by runtime or removed?
- Do we want realm binding columns for `client` and `docker` flows in the schema?
- Should flow version checksums be cryptographic hashes of the execution artifact, the graph JSON, or both?
- What is the desired policy for refresh-token SSO cookie lifetime and renewal?
- What is the minimum plugin security model required for early adopters?
