-- ============================================================================
-- ArchBot 数字员工系统 — DDL v2
-- 迁移编号: m20260529_001
-- 评审: DBA (dba-expert) + 架构师 (system-architect) ✅
-- ============================================================================

-- 1. 数字员工
CREATE TABLE IF NOT EXISTS digital_employees (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  code              VARCHAR(64)  NOT NULL UNIQUE,
  name              VARCHAR(255) NOT NULL,
  is_builtin        TINYINT      NOT NULL DEFAULT 1,
  avatar            VARCHAR(64)  NOT NULL DEFAULT '🤖',
  personality_tags  TEXT         NOT NULL DEFAULT '[]',
  personality_desc  TEXT         NOT NULL DEFAULT '',
  comm_style        VARCHAR(64)  NOT NULL DEFAULT 'formal',
  decision_pref     VARCHAR(64)  NOT NULL DEFAULT 'data_driven',
  focus_areas       TEXT         NOT NULL DEFAULT '[]',
  deliverable_groups TEXT        NOT NULL DEFAULT '[]',
  default_op        VARCHAR(64)  NOT NULL DEFAULT 'write',
  sort_order        INTEGER      NOT NULL DEFAULT 0,
  created_at        VARCHAR(255) NOT NULL,
  updated_at        VARCHAR(255) NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_de_builtin ON digital_employees(is_builtin);

-- 2. Skill
CREATE TABLE IF NOT EXISTS skills (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  code        VARCHAR(64)  NOT NULL UNIQUE,
  name        VARCHAR(255) NOT NULL,
  type        VARCHAR(64)  NOT NULL DEFAULT 'builtin',
  source_path TEXT         NOT NULL,
  description TEXT         NOT NULL DEFAULT '',
  created_at  VARCHAR(255) NOT NULL,
  updated_at  VARCHAR(255) NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_skill_code ON skills(code);

-- 3. Agent
CREATE TABLE IF NOT EXISTS agents (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  code        VARCHAR(64)  NOT NULL UNIQUE,
  name        VARCHAR(255) NOT NULL,
  type        VARCHAR(64)  NOT NULL DEFAULT 'builtin',
  source_path TEXT         NOT NULL,
  description TEXT         NOT NULL DEFAULT '',
  created_at  VARCHAR(255) NOT NULL,
  updated_at  VARCHAR(255) NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_code ON agents(code);

-- 4. MCP
CREATE TABLE IF NOT EXISTS mcps (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  code        VARCHAR(64)  NOT NULL UNIQUE,
  name        VARCHAR(255) NOT NULL,
  type        VARCHAR(64)  NOT NULL DEFAULT 'builtin',
  source_path TEXT         NOT NULL,
  description TEXT         NOT NULL DEFAULT '',
  created_at  VARCHAR(255) NOT NULL,
  updated_at  VARCHAR(255) NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_code ON mcps(code);

-- 5. 员工-Skill 关联
CREATE TABLE IF NOT EXISTS employee_skills (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  employee_code VARCHAR(64)  NOT NULL,
  skill_code    VARCHAR(64)  NOT NULL,
  created_at    VARCHAR(255) NOT NULL,
  UNIQUE (employee_code, skill_code),
  FOREIGN KEY (employee_code) REFERENCES digital_employees(code) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (skill_code)    REFERENCES skills(code)           ON DELETE CASCADE ON UPDATE CASCADE
);

-- 6. 员工-Agent 关联
CREATE TABLE IF NOT EXISTS employee_agents (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  employee_code VARCHAR(64)  NOT NULL,
  agent_code    VARCHAR(64)  NOT NULL,
  created_at    VARCHAR(255) NOT NULL,
  UNIQUE (employee_code, agent_code),
  FOREIGN KEY (employee_code) REFERENCES digital_employees(code) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (agent_code)    REFERENCES agents(code)            ON DELETE CASCADE ON UPDATE CASCADE
);

-- 7. 员工-MCP 关联
CREATE TABLE IF NOT EXISTS employee_mcps (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  employee_code VARCHAR(64)  NOT NULL,
  mcp_code      VARCHAR(64)  NOT NULL,
  created_at    VARCHAR(255) NOT NULL,
  UNIQUE (employee_code, mcp_code),
  FOREIGN KEY (employee_code) REFERENCES digital_employees(code) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (mcp_code)      REFERENCES mcps(code)              ON DELETE CASCADE ON UPDATE CASCADE
);

-- 8. 交互规则
CREATE TABLE IF NOT EXISTS employee_handoffs (
  id                   INTEGER PRIMARY KEY AUTOINCREMENT,
  employee_code        VARCHAR(64)  NOT NULL,
  trigger_op           VARCHAR(64)  NOT NULL,
  target_employee_code VARCHAR(64)  NOT NULL,
  transfer_data        TEXT         NOT NULL DEFAULT '[]',
  transfer_mode        VARCHAR(64)  NOT NULL DEFAULT 'notify',
  context_ref          TEXT         NOT NULL DEFAULT '',
  created_at           VARCHAR(255) NOT NULL,
  updated_at           VARCHAR(255) NOT NULL,
  UNIQUE (employee_code, trigger_op, target_employee_code),
  FOREIGN KEY (employee_code)        REFERENCES digital_employees(code) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (target_employee_code) REFERENCES digital_employees(code) ON DELETE CASCADE ON UPDATE CASCADE
);

-- 9. 审批队列
CREATE TABLE IF NOT EXISTS approval_queue (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  from_employee_code  VARCHAR(64)  NOT NULL,
  to_employee_code    VARCHAR(64)  NOT NULL,
  operation_type      VARCHAR(64)  NOT NULL,
  source_artifact     VARCHAR(256) NOT NULL,
  status              VARCHAR(64)  NOT NULL DEFAULT 'pending',
  result_data         TEXT         NOT NULL DEFAULT '',
  project_code        VARCHAR(64)  NOT NULL,
  created_at          VARCHAR(255) NOT NULL,
  updated_at          VARCHAR(255) NOT NULL,
  completed_at        VARCHAR(255) NOT NULL DEFAULT '',
  FOREIGN KEY (from_employee_code) REFERENCES digital_employees(code) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (to_employee_code)   REFERENCES digital_employees(code) ON DELETE CASCADE ON UPDATE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_aq_status    ON approval_queue(status);
CREATE INDEX IF NOT EXISTS idx_aq_to_status ON approval_queue(to_employee_code, status);
CREATE INDEX IF NOT EXISTS idx_aq_project   ON approval_queue(project_code);
