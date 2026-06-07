# 04 关系表述文件标准

## 一、结论

你发现的问题是对的。

最新版统一方法论里已有 `concept-relations.yml`，但它不足以完整表达以下关系：

```text
1. 概念之间的关系
2. 元能力之间的关系
3. 组合套路与元能力之间的关系
4. 枚举与 schema / workflow / policy / gate 之间的关系
5. 横切对象和主干层级之间的关系
6. 认知型能力与反馈演化对象之间的关系
7. 本质抽象、图、迁移之间的关系
```

所以，不是你错，是之前的关系文件不完整。

应该补一个统一的关系注册文件：

```text
relationship-registry.yml
```

它不替代 `concept-relations.yml`，而是覆盖更完整的关系层。

---

## 二、关系文件分层

建议分成 6 类关系文件：

```text
01_concept_relations.yml
02_capability_relations.yml
03_pattern_relations.yml
04_enum_relations.yml
05_cross_cutting_relations.yml
06_evolution_relations.yml
```

如果不想拆多个文件，也可以合并为：

```text
relationship-registry.yml
```

但内部必须分 section。

---

## 三、关系类型命名空间

```yaml
relation_type_namespaces:
  concept_relation_types:
    - contains
    - depends_on
    - produces
    - validates
    - controls
    - references
    - blocks
    - resolves

  capability_relation_types:
    - depends_on
    - produces
    - consumes
    - invokes
    - validates
    - controls
    - enriches

  pattern_relation_types:
    - composed_of
    - ordered_before
    - requires
    - produces
    - handles_failure_with

  enum_relation_types:
    - constrains
    - validates
    - routes
    - classifies
    - used_by

  cross_cutting_relation_types:
    - crosses
    - constrains
    - enriches
    - monitors

  evolution_relation_types:
    - observes
    - abstracts
    - reinforces
    - weakens
    - distills_into
    - updates
```

---

## 四、统一关系记录格式

所有关系都必须使用完整边结构：

```yaml
relation:
  id: ""
  namespace: ""
  type: ""
  from:
    kind: "concept | capability | pattern | enum | runtime_object | evolution_object"
    id: ""
  to:
    kind: "concept | capability | pattern | enum | runtime_object | evolution_object"
    id: ""
  meaning: ""
  evidence_required: true
  enforcement: "schema | runtime | coordinator | human | review"
  validation_rules: []
```

禁止省略 `from`。  
禁止只写 `target`。  
禁止让上下文隐含关系方向。

---

## 五、关系正确性标准

每条关系必须通过：

```text
1. from.kind 合法
2. from.id 已定义
3. to.kind 合法
4. to.id 已定义
5. namespace 合法
6. type 属于 namespace
7. 方向正确
8. meaning 与方向一致
9. 有 evidence 或规则依据
10. 可被 schema / runtime / review 使用
```

---

## 六、推荐关系文件结构

```yaml
relationship_registry:
  version: "2.1"

  concept_relations: []
  capability_relations: []
  pattern_relations: []
  enum_relations: []
  cross_cutting_relations: []
  evolution_relations: []
```

---

## 七、必须补齐的关系类型

### 1. 概念关系

```text
workflow depends_on required_inputs
stage_gates validates workflow
execution_rules controls workflow
review_checklist validates deliverables
file_structure contains context_files
fallback_rules controls workflow
decision_rules resolves context_files
```

### 2. 元能力关系

```text
situation_recognition depends_on environment_recognition
strategy_selection depends_on situation_recognition
concept_extraction depends_on input_capability
relation_capability depends_on concept_capability
workflow_capability depends_on gate_capability
artifact_capability depends_on file_contract_capability
feedback_learning_loop depends_on audit_capability
essence_abstraction_and_transfer depends_on historical_reflection
```

### 3. 组合套路关系

```text
concept_extraction_pattern composed_of llm_guard_capability
concept_extraction_pattern composed_of concept_capability
concept_extraction_pattern composed_of schema_capability

stage_gate_pattern composed_of gate_capability
stage_gate_pattern composed_of review_capability
stage_gate_pattern composed_of fallback_capability
```

### 4. 枚举关系

```text
owner_type constrains owners
relation_type validates concept_relations.type
rule_effect constrains policy.effect
gate_result constrains gate.result
severity constrains review_checklist.severity
update_mode constrains methodology_versioning.update_modes
```

### 5. 横切关系

```text
domain_profile crosses input_layer
domain_profile crosses rule_layer
evidence_traceability crosses context_layer
confidence_policy crosses review_layer
essence_abstraction_and_transfer crosses context_layer
```

### 6. 演化关系

```text
audit_logs observes runtime_execution
historical_reflection consumes audit_logs
pattern_induction abstracts reflection
methodology_distillation distills_into methodology_update_candidate
feedback_learning_loop updates methodology_version_record
```
