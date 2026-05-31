-- ============================================================================
-- ArchBot Agent 控制平面 — DDL v1
-- 迁移编号: m20260531_001
-- 10 张表: agent_runtime, agent_runtime_version, agent_adapter,
--          agent_session, agent_turn, agent_event, agent_artifact,
--          agent_file_change, agent_decision, agent_context_snapshot,
--          agent_audit_log
-- ============================================================================

-- 1. Agent Runtime 注册表
CREATE TABLE IF NOT EXISTS agent_runtime (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  runtime_type    VARCHAR(64)  NOT NULL UNIQUE,
  enabled         TINYINT      NOT NULL DEFAULT 1,
  mode            VARCHAR(64)  NOT NULL DEFAULT 'managed',
  current_version VARCHAR(64)  NOT NULL DEFAULT '',
  executable_path TEXT         NOT NULL DEFAULT '',
  adapter_config  TEXT         NOT NULL DEFAULT '{}',
  provider_config TEXT         NOT NULL DEFAULT '{}',
  model_config    TEXT         NOT NULL DEFAULT '{}',
  env_vars        TEXT         NOT NULL DEFAULT '{}',
  execution_config TEXT        NOT NULL DEFAULT '{}',
  created_at      VARCHAR(255) NOT NULL,
  updated_at      VARCHAR(255) NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ar_type ON agent_runtime(runtime_type);
CREATE INDEX IF NOT EXISTS idx_ar_enabled ON agent_runtime(enabled);

-- 2. Runtime 版本记录表
CREATE TABLE IF NOT EXISTS agent_runtime_version (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  runtime_id    INTEGER      NOT NULL,
  version       VARCHAR(64)  NOT NULL,
  install_path  TEXT         NOT NULL DEFAULT '',
  checksum      VARCHAR(128) NOT NULL DEFAULT '',
  status        VARCHAR(32)  NOT NULL DEFAULT 'installed',
  installed_at  VARCHAR(255) NOT NULL,
  created_at    VARCHAR(255) NOT NULL,
  FOREIGN KEY (runtime_id) REFERENCES agent_runtime(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_arv_runtime ON agent_runtime_version(runtime_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_arv_runtime_ver ON agent_runtime_version(runtime_id, version);

-- 3. Adapter 进程记录表
CREATE TABLE IF NOT EXISTS agent_adapter (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  runtime_id       INTEGER      NOT NULL,
  adapter_type     VARCHAR(64)  NOT NULL,
  host             VARCHAR(255) NOT NULL DEFAULT '127.0.0.1',
  port             INTEGER      NOT NULL DEFAULT 0,
  auth_token_hash  VARCHAR(255) NOT NULL DEFAULT '',
  status           VARCHAR(32)  NOT NULL DEFAULT 'stopped',
  pid              INTEGER      NOT NULL DEFAULT 0,
  started_at       VARCHAR(255) NOT NULL DEFAULT '',
  stopped_at       VARCHAR(255) NOT NULL DEFAULT '',
  FOREIGN KEY (runtime_id) REFERENCES agent_runtime(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_aa_runtime ON agent_adapter(runtime_id);

-- 4. 会话表
CREATE TABLE IF NOT EXISTS agent_session (
  session_id     VARCHAR(64)  PRIMARY KEY,
  title          VARCHAR(255) NOT NULL,
  goal           TEXT         NOT NULL DEFAULT '',
  project_id     VARCHAR(128) NOT NULL DEFAULT '',
  runtime_type   VARCHAR(64)  NOT NULL DEFAULT '',
  default_model  VARCHAR(128) NOT NULL DEFAULT '',
  current_state  TEXT         NOT NULL DEFAULT '',
  status         VARCHAR(32)  NOT NULL DEFAULT 'active',
  created_at     VARCHAR(255) NOT NULL,
  updated_at     VARCHAR(255) NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_as_project ON agent_session(project_id);
CREATE INDEX IF NOT EXISTS idx_as_status ON agent_session(status);

-- 5. Turn 表
CREATE TABLE IF NOT EXISTS agent_turn (
  turn_id            VARCHAR(64)  PRIMARY KEY,
  session_id         VARCHAR(64)  NOT NULL,
  sequence_number    INTEGER      NOT NULL DEFAULT 0,
  user_message       TEXT         NOT NULL DEFAULT '',
  interpreted_intent TEXT         NOT NULL DEFAULT '',
  input_file_path    TEXT         NOT NULL DEFAULT '',
  prompt_file_path   TEXT         NOT NULL DEFAULT '',
  status             VARCHAR(32)  NOT NULL DEFAULT 'pending',
  runtime_type       VARCHAR(64)  NOT NULL DEFAULT '',
  runtime_version    VARCHAR(64)  NOT NULL DEFAULT '',
  model              VARCHAR(128) NOT NULL DEFAULT '',
  started_at         VARCHAR(255) NOT NULL DEFAULT '',
  finished_at        VARCHAR(255) NOT NULL DEFAULT '',
  error_message      TEXT         NOT NULL DEFAULT '',
  duration_ms        INTEGER      NOT NULL DEFAULT 0,
  FOREIGN KEY (session_id) REFERENCES agent_session(session_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_at_session ON agent_turn(session_id);
CREATE INDEX IF NOT EXISTS idx_at_status ON agent_turn(status);

-- 6. 事件表
CREATE TABLE IF NOT EXISTS agent_event (
  event_id        VARCHAR(64)  PRIMARY KEY,
  session_id      VARCHAR(64)  NOT NULL,
  turn_id         VARCHAR(64)  NOT NULL,
  event_type      VARCHAR(64)  NOT NULL,
  sequence_number INTEGER      NOT NULL DEFAULT 0,
  payload         TEXT         NOT NULL DEFAULT '{}',
  timestamp       VARCHAR(255) NOT NULL,
  FOREIGN KEY (turn_id) REFERENCES agent_turn(turn_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ae_turn ON agent_event(turn_id);
CREATE INDEX IF NOT EXISTS idx_ae_turn_seq ON agent_event(turn_id, sequence_number);

-- 7. 产物表
CREATE TABLE IF NOT EXISTS agent_artifact (
  artifact_id   VARCHAR(64)  PRIMARY KEY,
  turn_id       VARCHAR(64)  NOT NULL,
  artifact_type VARCHAR(64)  NOT NULL DEFAULT '',
  file_path     TEXT         NOT NULL DEFAULT '',
  mime_type     VARCHAR(128) NOT NULL DEFAULT '',
  size_bytes    INTEGER      NOT NULL DEFAULT 0,
  created_at    VARCHAR(255) NOT NULL,
  FOREIGN KEY (turn_id) REFERENCES agent_turn(turn_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_aart_turn ON agent_artifact(turn_id);

-- 8. 文件变更表
CREATE TABLE IF NOT EXISTS agent_file_change (
  change_id      VARCHAR(64)  PRIMARY KEY,
  turn_id        VARCHAR(64)  NOT NULL,
  file_path      TEXT         NOT NULL,
  change_type    VARCHAR(32)  NOT NULL DEFAULT 'modified',
  diff_content   TEXT         NOT NULL DEFAULT '',
  file_hash_before VARCHAR(128) NOT NULL DEFAULT '',
  file_hash_after  VARCHAR(128) NOT NULL DEFAULT '',
  size_before    INTEGER      NOT NULL DEFAULT 0,
  size_after     INTEGER      NOT NULL DEFAULT 0,
  created_at     VARCHAR(255) NOT NULL,
  FOREIGN KEY (turn_id) REFERENCES agent_turn(turn_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_afc_turn ON agent_file_change(turn_id);

-- 9. 决策表
CREATE TABLE IF NOT EXISTS agent_decision (
  decision_id   VARCHAR(64)  PRIMARY KEY,
  session_id    VARCHAR(64)  NOT NULL,
  turn_id       VARCHAR(64),
  decision_text TEXT         NOT NULL DEFAULT '',
  status        VARCHAR(32)  NOT NULL DEFAULT 'proposed',
  rationale     TEXT         NOT NULL DEFAULT '',
  created_at    VARCHAR(255) NOT NULL,
  superseded_by VARCHAR(64)  DEFAULT NULL,
  FOREIGN KEY (session_id) REFERENCES agent_session(session_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ad_session ON agent_decision(session_id);

-- 10. 上下文快照表
CREATE TABLE IF NOT EXISTS agent_context_snapshot (
  snapshot_id   VARCHAR(64)  PRIMARY KEY,
  session_id    VARCHAR(64)  NOT NULL,
  turn_id       VARCHAR(64),
  snapshot_type VARCHAR(64)  NOT NULL DEFAULT 'pre_turn',
  content       TEXT         NOT NULL DEFAULT '{}',
  token_count   INTEGER      NOT NULL DEFAULT 0,
  created_at    VARCHAR(255) NOT NULL,
  FOREIGN KEY (session_id) REFERENCES agent_session(session_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_acs_session ON agent_context_snapshot(session_id);

-- 11. 审计日志表
CREATE TABLE IF NOT EXISTS agent_audit_log (
  log_id      VARCHAR(64)  PRIMARY KEY,
  session_id  VARCHAR(64)  NOT NULL DEFAULT '',
  turn_id     VARCHAR(64)  NOT NULL DEFAULT '',
  event_type  VARCHAR(64)  NOT NULL DEFAULT '',
  actor       VARCHAR(64)  NOT NULL DEFAULT '',
  action      VARCHAR(128) NOT NULL DEFAULT '',
  detail      TEXT         NOT NULL DEFAULT '{}',
  severity    VARCHAR(32)  NOT NULL DEFAULT 'info',
  created_at  VARCHAR(255) NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_aal_severity ON agent_audit_log(severity);
CREATE INDEX IF NOT EXISTS idx_aal_session ON agent_audit_log(session_id);
CREATE INDEX IF NOT EXISTS idx_aal_created ON agent_audit_log(created_at);
