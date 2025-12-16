-- Stores the last JSON instruction sent to the UI (e.g. the Form Schema).
-- This allows the frontend to reload the page without re-triggering 'execute' logic.
ALTER TABLE auth_sessions
    ADD COLUMN last_ui_output TEXT;

-- Tracks if the current node is waiting for input or async events.
-- Values: 'idle', 'waiting_for_input', 'waiting_for_async', 'completed'
ALTER TABLE auth_sessions
    ADD COLUMN execution_state TEXT NOT NULL DEFAULT 'idle';