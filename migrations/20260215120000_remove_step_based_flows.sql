-- Remove legacy step-based flow tables
DROP TABLE IF EXISTS auth_flow_steps;
DROP TABLE IF EXISTS authenticator_config;
DROP TABLE IF EXISTS login_sessions;
