-- ============================================================================
-- ArchBot 数字员工系统 — 内置种子数据
-- 18 个内置员工 + 关联的 skills / agents / mcps
-- ============================================================================

-- ── 18 个内置数字员工 ──────────────────────────────────────────────
INSERT INTO digital_employees (code, name, is_builtin, personality_tags, focus_areas, deliverable_groups, default_op, sort_order, created_at, updated_at) VALUES
('ba-analyst',           '需求分析师',   1, '["严谨","逻辑","归纳"]',         '["需求"]',             '["bizContext","userStories"]',                        'write',  1,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('po-decision-maker',   '产品决策官',   1, '["商业","果断","战略"]',         '["需求"]',             '["bizContext","userStories","funcSpec","qualityAttr"]', 'review', 2,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('business-sponsor',    '业务发起人',   1, '["高层","愿景","务实"]',         '["需求"]',             '["bizContext"]',                                        'write',  3,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('domain-sme',          '领域专家',     1, '["专精","细节","经验"]',         '["需求"]',             '["bizContext","bizRules","dataStandard"]',              'review', 4,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('project-coordinator', '项目协调员',   1, '["有序","守时","沟通"]',         '["需求","测试","部署"]',  '[]',                                                  'review', 5,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('system-architect',    '系统架构师',   1, '["工程","全局","权衡"]',         '["设计","开发"]',        '["funcSpec","qualityAttr"]',                            'write',  6,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('security-officer',    '安全合规官',   1, '["风险","缜密","零容忍"]',       '["设计","测试"]',        '["qualityAttr","dataClassify"]',                         'write',  7,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('data-modeler',        '数据建模师',   1, '["抽象","范式","系统"]',         '["需求","设计"]',        '["dataModel","dataDictionary"]',                        'write',  8,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('dba-expert',          '数据库专家',   1, '["务实","精确","性能"]',         '["设计"]',              '["dataModel"]',                                         'write',  9,  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('data-steward',        '数据治理员',   1, '["标准","一致","元数据"]',       '["设计"]',              '["dataDictionary","dataClassify"]',                      'write',  10, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('compliance-officer',  '法务合规官',   1, '["严谨","法规","制衡"]',         '["需求","测试"]',        '["bizRules","qualityAttr"]',                             'review', 11, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('ux-designer',         '交互设计师',   1, '["用户","审美","同理"]',         '["设计"]',              '["prototype","funcSpec"]',                               'write',  12, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('frontend-dev',        '前端工程师',   1, '["像素","交互","性能"]',         '["开发"]',              '["prototype"]',                                         'write',  13, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('backend-dev',         '后端工程师',   1, '["逻辑","契约","健壮"]',         '["开发"]',              '["apiContract","integration"]',                          'write',  14, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('test-engineer',       '测试工程师',   1, '["刨根","覆盖","自动化"]',       '["测试"]',              '["qualityAttr","funcSpec"]',                             'review', 15, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('sre-engineer',        '运维专家',     1, '["稳定","监控","容灾"]',         '["部署"]',              '["qualityAttr","migration"]',                            'write',  16, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('etl-engineer',        '数据工程师',   1, '["管道","转换","增量"]',         '["开发"]',              '["dataFlow","dataModel"]',                               'write',  17, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('external-coordinator','外部协调员',   1, '["协议","适配","外部"]',         '["设计","开发"]',        '["integration","apiContract"]',                          'write',  18, '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00');

-- ── 内置 Skills ────────────────────────────────────────────────
INSERT INTO skills (code, name, type, source_path, description, created_at, updated_at) VALUES
('biz-process-extract',  '业务流程提取',   'builtin', 'resource://skills/biz-process-extract/',  '从BPMN流程图中提取活动节点和决策网关，生成Story和业务规则', '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('story-generation',     '故事生成',       'builtin', 'resource://skills/story-generation/',     '从业务流程图自动生成用户故事，附带验收标准',               '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('entity-recognition',   '实体识别',       'builtin', 'resource://skills/entity-recognition/',   '从业务流程和业务规则中识别候选实体和属性',                 '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('normalization-check',  '范式检查',       'builtin', 'resource://skills/normalization-check/',  '检查数据模型的3NF合规性，标注反范式化点',                  '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('ddl-generator',        'DDL生成',        'builtin', 'resource://skills/ddl-generator/',        '从物理数据模型生成CREATE TABLE/INDEX DDL脚本',             '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('api-contract-gen',     'API契约生成',    'builtin', 'resource://skills/api-contract-gen/',     '从数据模型和功能树生成OpenAPI规范',                       '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('prototype-gen',        '原型生成',       'builtin', 'resource://skills/prototype-gen/',        '从功能描述和Story生成HTML原型页面',                       '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('stride-threat-model',  'STRIDE威胁建模', 'builtin', 'resource://skills/stride-threat-model/',  '对系统执行STRIDE六类威胁建模',                            '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('sla-matrix-gen',       'SLA矩阵生成',    'builtin', 'resource://skills/sla-matrix-gen/',       '根据业务重要性自动生成分级SLA矩阵',                       '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('compliance-mapping',   '法规对标',       'builtin', 'resource://skills/compliance-mapping/',   '将法规要求逐条映射到技术控制措施',                         '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('reverse-engineering',  '逆向工程',       'builtin', 'resource://skills/reverse-engineering/',  '从DDL/数据库/代码反向生成数据模型',                        '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00');

-- ── 内置 Agents ────────────────────────────────────────────────
INSERT INTO agents (code, name, type, source_path, description, created_at, updated_at) VALUES
('code-reviewer',   '代码审查',   'builtin', 'resource://agents/code-reviewer/',   '代码质量审查，发现缺陷和优化机会',           '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('architect',       '架构师',     'builtin', 'resource://agents/architect/',       '系统架构设计和评审',                         '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('tdd-guide',       'TDD引导',    'builtin', 'resource://agents/tdd-guide/',       '测试驱动开发引导，保障80%+覆盖率',            '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('security-reviewer','安全审查',  'builtin', 'resource://agents/security-reviewer/','安全漏洞检测和渗透测试',                     '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('planner',         '规划师',     'builtin', 'resource://agents/planner/',         '实现计划和任务分解',                         '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00');

-- ── 内置 MCPs ──────────────────────────────────────────────────
INSERT INTO mcps (code, name, type, source_path, description, created_at, updated_at) VALUES
('context7',        'Context7 文档',      'builtin', 'resource://mcps/context7/',        '实时查询编程框架和库的最新文档',     '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('playwright',      'Playwright 浏览器',  'builtin', 'resource://mcps/playwright/',      '浏览器自动化和E2E测试',             '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('chrome-devtools', 'Chrome DevTools',    'builtin', 'resource://mcps/chrome-devtools/',  'Chrome开发者工具集成',              '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00'),
('postman',         'Postman API',        'builtin', 'resource://mcps/postman/',         'API测试和集合管理',                 '2026-05-29T00:00:00+08:00', '2026-05-29T00:00:00+08:00');

-- ── 员工-Skill 关联 (按角色职责分配) ───────────────────────────
-- BA: 流程提取 + 故事生成
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('ba-analyst', 'biz-process-extract', '2026-05-29T00:00:00+08:00'),
('ba-analyst', 'story-generation',    '2026-05-29T00:00:00+08:00');
-- PO: 故事生成
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('po-decision-maker', 'story-generation', '2026-05-29T00:00:00+08:00');
-- 数据建模师: 实体识别 + 范式检查
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('data-modeler', 'entity-recognition',  '2026-05-29T00:00:00+08:00'),
('data-modeler', 'normalization-check', '2026-05-29T00:00:00+08:00');
-- DBA: DDL生成 + 范式检查
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('dba-expert', 'ddl-generator',        '2026-05-29T00:00:00+08:00'),
('dba-expert', 'normalization-check',  '2026-05-29T00:00:00+08:00');
-- 架构师: API契约生成 + DDL生成
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('system-architect', 'api-contract-gen', '2026-05-29T00:00:00+08:00'),
('system-architect', 'ddl-generator',    '2026-05-29T00:00:00+08:00');
-- UX: 原型生成
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('ux-designer', 'prototype-gen', '2026-05-29T00:00:00+08:00');
-- 安全合规官: STRIDE + 法规对标
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('security-officer', 'stride-threat-model', '2026-05-29T00:00:00+08:00'),
('security-officer', 'compliance-mapping',  '2026-05-29T00:00:00+08:00');
-- SRE: SLA矩阵
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('sre-engineer', 'sla-matrix-gen', '2026-05-29T00:00:00+08:00');
-- 外部协调员 + ETL: 逆向工程
INSERT INTO employee_skills (employee_code, skill_code, created_at) VALUES
('external-coordinator', 'reverse-engineering', '2026-05-29T00:00:00+08:00'),
('etl-engineer',         'reverse-engineering', '2026-05-29T00:00:00+08:00');

-- ── 员工-Agent 关联 ────────────────────────────────────────────
INSERT INTO employee_agents (employee_code, agent_code, created_at) VALUES
('system-architect', 'architect',        '2026-05-29T00:00:00+08:00'),
('system-architect', 'code-reviewer',    '2026-05-29T00:00:00+08:00'),
('backend-dev',      'code-reviewer',    '2026-05-29T00:00:00+08:00'),
('frontend-dev',     'code-reviewer',    '2026-05-29T00:00:00+08:00'),
('test-engineer',    'tdd-guide',        '2026-05-29T00:00:00+08:00'),
('security-officer', 'security-reviewer','2026-05-29T00:00:00+08:00'),
('project-coordinator','planner',        '2026-05-29T00:00:00+08:00');

-- ── 员工-MCP 关联 ──────────────────────────────────────────────
INSERT INTO employee_mcps (employee_code, mcp_code, created_at) VALUES
('ba-analyst',      'context7',       '2026-05-29T00:00:00+08:00'),
('system-architect','context7',       '2026-05-29T00:00:00+08:00'),
('backend-dev',     'postman',        '2026-05-29T00:00:00+08:00'),
('test-engineer',   'playwright',     '2026-05-29T00:00:00+08:00'),
('frontend-dev',    'chrome-devtools','2026-05-29T00:00:00+08:00'),
('ux-designer',     'playwright',     '2026-05-29T00:00:00+08:00');
