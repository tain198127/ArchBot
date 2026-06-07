# 02 组合套路判定标准

## 一、定义

组合套路是指：

> 多个元能力在稳定场景下，按固定顺序或固定协作关系组合，用于解决一类重复出现的问题的执行模式。

组合套路不是单个能力，也不是普通流程阶段。

---

## 二、必须成为组合套路的条件

一个候选对象同时满足以下条件，才应该提炼为组合套路。

| 判断项 | 标准 |
|---|---|
| 多能力组合 | 至少组合 3 个元能力 |
| 顺序稳定 | 有相对稳定的执行顺序或协作结构 |
| 场景复用 | 能在多个领域或多个 workflow 中复用 |
| 问题稳定 | 解决的是一类重复出现的问题 |
| 有输入输出 | 能定义套路级输入和输出 |
| 有门禁 | 能定义进入条件和退出条件 |
| 有失败处理 | 能定义失败后怎么办 |
| 可被解释器实现 | 可以对应 PatternRunner / Engine |
| 可审查 | 能判断是否执行成功 |

至少满足 7 项，才可成为组合套路。

---

## 三、不应该成为组合套路的情况

| 类型 | 原因 |
|---|---|
| 只有一个动作 | 应是元能力或普通 task |
| 只是自然语言步骤 | 不稳定、不可执行 |
| 只在一个项目用一次 | 复用性不足 |
| 没有稳定顺序 | 不适合套路化 |
| 没有进入/退出条件 | 不可运行 |
| 没有失败处理 | 不可靠 |
| 只是业务模板 | 应放 templates |
| 只是审查清单 | 应放 review_checklist |

---

## 四、组合套路判定问题

```text
1. 它是否组合了多个元能力？
2. 这些元能力是否有稳定协作顺序？
3. 它是否解决一类重复问题？
4. 它是否可跨领域复用？
5. 它是否有进入条件？
6. 它是否有退出条件？
7. 它是否有失败路径？
8. 它是否能对应解释器？
9. 它是否能被审查？
10. 它是否只是 workflow 的某个阶段？
```

---

## 五、评分标准

| 维度 | 分值 |
|---|---:|
| 多能力组合程度 | 15 |
| 顺序稳定性 | 15 |
| 复用性 | 15 |
| 问题类型稳定 | 15 |
| 输入输出清晰 | 10 |
| 门禁清晰 | 10 |
| 失败处理清晰 | 10 |
| 可实现为解释器 | 10 |
| 总分 | 100 |

判定：

```text
90-100：应成为组合套路
75-89：可成为组合套路，但需补门禁或失败路径
60-74：暂列候选套路
60 以下：不应成为组合套路
```

---

## 六、组合套路模板

```yaml
composition_pattern:
  id: ""
  name: ""
  problem: ""
  applicable_when: []
  not_applicable_when: []
  capabilities:
    - ""
  execution_order:
    - step: ""
      capability: ""
      input: []
      output: []
  entry_conditions: []
  exit_conditions: []
  failure_paths: []
  used_by_workflows: []
  interpreter: ""
  review_checklist: []
```

---

## 七、统一方法论中的基础组合套路

### 执行组合套路

```text
input_validation_pattern
concept_extraction_pattern
relation_modeling_pattern
policy_guard_pattern
stage_gate_pattern
state_machine_pattern
artifact_generation_pattern
fallback_pattern
```

### 认知组合套路

```text
environment_situation_recognition_pattern
contextual_insight_pattern
reflection_pattern_induction_pattern
knowledge_transfer_pattern
feedback_evolution_pattern
essence_abstraction_transfer_pattern
```
