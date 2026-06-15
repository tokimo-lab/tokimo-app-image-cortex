CREATE TABLE IF NOT EXISTS system_config (
    scope      text        NOT NULL,
    scope_id   text        NOT NULL,
    value      jsonb       NOT NULL DEFAULT '{}',
    updated_at timestamptz NOT NULL DEFAULT NOW(),
    PRIMARY KEY (scope, scope_id)
);
