-- Business Flow Designer — 3-table schema
-- Migration: m20260604_001
-- Creates: business_flows, flow_runs, flow_run_artifacts

-- ─── business_flows ──────────────────────────────────────────
CREATE TABLE IF NOT EXISTS business_flows (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL UNIQUE,
    description  TEXT NOT NULL DEFAULT '',
    type         TEXT NOT NULL DEFAULT 'custom' CHECK (type IN ('builtin', 'custom')),
    published    INTEGER NOT NULL DEFAULT 0 CHECK (published IN (0, 1)),
    flow_json    TEXT NOT NULL DEFAULT '{}',
    output_dir   TEXT NOT NULL DEFAULT './output',
    output_filename_pattern TEXT NOT NULL DEFAULT '{flow}_{date}',
    output_extension        TEXT NOT NULL DEFAULT '.md',
    scenario_bindings       TEXT NOT NULL DEFAULT '[]',
    yaml_export             TEXT,
    created_at              TEXT NOT NULL,
    updated_at              TEXT NOT NULL,
    published_at            TEXT,
    version                 INTEGER NOT NULL DEFAULT 1 CHECK (version > 0)
);

CREATE INDEX IF NOT EXISTS idx_business_flows_type ON business_flows(type);
CREATE INDEX IF NOT EXISTS idx_business_flows_published ON business_flows(published);

-- ─── flow_runs ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS flow_runs (
    id              TEXT PRIMARY KEY NOT NULL,
    flow_id         TEXT NOT NULL REFERENCES business_flows(id) ON DELETE CASCADE,
    status          TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'running', 'completed', 'failed', 'aborted')),
    triggered_by    TEXT NOT NULL DEFAULT 'manual'
                    CHECK (triggered_by IN ('menu', 'manual', 'api')),
    material_paths  TEXT NOT NULL DEFAULT '[]',
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    output_log      TEXT NOT NULL DEFAULT '',
    error_message   TEXT
);

CREATE INDEX IF NOT EXISTS idx_flow_runs_flow_id ON flow_runs(flow_id);
CREATE INDEX IF NOT EXISTS idx_flow_runs_status ON flow_runs(status);

-- ─── flow_run_artifacts ──────────────────────────────────────
CREATE TABLE IF NOT EXISTS flow_run_artifacts (
    id              TEXT PRIMARY KEY NOT NULL,
    run_id          TEXT NOT NULL REFERENCES flow_runs(id) ON DELETE CASCADE,
    node_id         TEXT NOT NULL,
    agent_id        TEXT NOT NULL DEFAULT '',
    artifact_path   TEXT NOT NULL,
    artifact_type   TEXT NOT NULL DEFAULT 'file',
    created_at      TEXT NOT NULL,
    checksum        TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_flow_run_artifacts_run_id ON flow_run_artifacts(run_id);
