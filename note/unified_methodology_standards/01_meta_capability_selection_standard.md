# 01 元能力判定标准

## 一、定义

元能力是指：

> 在多个方法论、多个领域或多个执行场景中可复用的基础执行能力或认知能力。

元能力不是业务步骤，也不是具体任务，也不是某个文件。

---

## 二、必须成为元能力的条件

一个候选对象同时满足以下条件，才应该提炼为元能力。

| 判断项 | 标准 |
|---|---|
| 可复用 | 能在多个 workflow、多个领域、多个 skill 中复用 |
| 有输入 | 能明确输入对象 |
| 有输出 | 能明确输出对象 |
| 有执行语义 | 能说明它做什么、怎么做、何时完成 |
| 可被编排 | 能被 workflow / pattern 调用 |
| 可被校验 | 能定义成功 / 失败 / 异常 |
| 有边界 | 能说明它不负责什么 |
| 可实现 | 能对应 engine / validator / analyzer / guard |
| 可审计 | 能记录执行结果和依据 |

至少满足 7 项，才可作为元能力。

---

## 三、不应该成为元能力的情况

以下内容不应该提炼为元能力：

| 类型 | 原因 |
|---|---|
| 一次性业务动作 | 复用性不足 |
| 某个具体文件 | 文件是资源，不是能力 |
| 某个具体字段 | 字段是数据，不是能力 |
| 某个口号 | 不可执行 |
| 某个模糊判断 | 无输入输出，不可校验 |
| 某个领域专属规则 | 应放 domain_profile 或 policy |
| 某个流程阶段名称 | 通常是 workflow state，不是能力 |
| 某个产物名称 | 应放 deliverables，不是能力 |

---

## 四、元能力分类

统一方法论中的元能力分两类：

```text
1. 执行型元能力
2. 认知型元能力
```

### 1. 执行型元能力

解决：

```text
如何稳定执行？
如何校验？
如何阻断？
如何写入？
如何审计？
```

典型例子：

```text
input_capability
schema_capability
workflow_capability
policy_capability
gate_capability
file_contract_capability
artifact_capability
audit_capability
```

### 2. 认知型元能力

解决：

```text
如何识别环境？
如何判断情景？
如何生成洞察？
如何提炼规律？
如何迁移知识？
如何抽象本质？
```

典型例子：

```text
environment_recognition
situation_recognition
contextual_insight_generation
pattern_induction
knowledge_transfer
essence_abstraction_and_transfer
```

---

## 五、元能力判定问题

判断候选对象是否为元能力时，必须问：

```text
1. 它是否能在多个场景复用？
2. 它是否有明确输入？
3. 它是否有明确输出？
4. 它是否能被 workflow 或组合套路调用？
5. 它是否能独立失败？
6. 它是否能被 engine 实现？
7. 它是否能被审计？
8. 它是否和已有元能力重复？
9. 它是否只是一个流程阶段？
10. 它是否只是一个业务规则？
```

---

## 六、评分标准

| 维度 | 分值 |
|---|---:|
| 复用性 | 15 |
| 输入输出清晰 | 15 |
| 执行语义明确 | 15 |
| 可编排 | 15 |
| 可校验 | 10 |
| 可实现 | 10 |
| 边界清晰 | 10 |
| 不重复 | 10 |
| 总分 | 100 |

判定：

```text
90-100：应成为元能力
75-89：可以成为元能力，但需补边界或输入输出
60-74：暂列候选能力
60 以下：不应成为元能力
```

---

## 七、元能力卡模板

```yaml
capability:
  id: ""
  type: "execution | cognitive"
  definition: ""
  not:
    - ""
  input_objects: []
  output_objects: []
  depends_on: []
  used_by_patterns: []
  used_by_engines: []
  llm_role: "none | propose | assist | explain"
  machine_role: ""
  human_review_required: true
  failure_modes: []
  validation_rules: []
  audit_events: []
```
