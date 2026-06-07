# 03 枚举提炼标准

## 一、定义

枚举是指：

> 系统需要稳定识别、校验、路由、阻断、选择或执行的有限值集合。

不是所有分类都应该提炼为枚举。  
只有会影响机器判断和执行的分类，才应该枚举化。

---

## 二、应该提炼为枚举的条件

| 判断项 | 标准 |
|---|---|
| 有限集合 | 值域有限，不是无限开放文本 |
| 稳定使用 | 多处被引用 |
| 影响执行 | 影响 workflow、policy、gate、schema 或 routing |
| 需要校验 | 非法值会导致错误 |
| 需要路由 | 不同值对应不同流程 |
| 需要统计 | 需要审计、报表、反馈分析 |
| 需要跨文件一致 | 多个配置文件必须保持一致 |

至少满足 5 项，才应该枚举化。

---

## 三、不应该提炼为枚举的情况

| 类型 | 原因 |
|---|---|
| 开放描述文本 | 值域不可穷举 |
| 一次性标签 | 复用性不足 |
| 临时项目名称 | 不稳定 |
| 业务自由文本 | 不能固定枚举 |
| 高度主观判断 | 容易误伤 |
| 只用于展示 | 不影响执行 |

---

## 四、必须枚举化的对象

统一方法论中以下对象应该枚举化：

```text
object_type
concept_layer
concept_id
relation_type
relation_namespace
capability_type
pattern_type
workflow_state_type
gate_result
rule_effect
severity
llm_role
owner_type
permission_type
confirmation_type
confidence_level
fallback_action
update_mode
risk_level
resource_kind
action_type
```

---

## 五、不应枚举化的对象

以下内容不应强制枚举：

```text
mission 文本
scope 自然语言说明
source_text
derived_reason
human_comment
evidence excerpt
artifact content
open question content
business-specific free text
```

---

## 六、枚举判定问题

```text
1. 这个值是否有限？
2. 是否会被多个文件引用？
3. 是否影响执行路径？
4. 是否需要校验非法值？
5. 是否会被 policy / gate / workflow 使用？
6. 是否需要统计和审计？
7. 是否跨项目稳定？
8. 是否允许业务自由扩展？
```

---

## 七、评分标准

| 维度 | 分值 |
|---|---:|
| 值域有限 | 20 |
| 执行影响 | 20 |
| 多处引用 | 15 |
| 需要校验 | 15 |
| 需要路由 | 10 |
| 跨文件一致性 | 10 |
| 稳定性 | 10 |
| 总分 | 100 |

判定：

```text
85-100：必须枚举化
70-84：建议枚举化
50-69：可作为开放分类
50 以下：不应枚举化
```

---

## 八、枚举模板

```yaml
enum:
  id: ""
  description: ""
  values:
    - id: ""
      name: ""
      description: ""
      allowed_in: []
      deprecated: false
  validation:
    allow_unknown: false
    unknown_value_policy: "reject | warn | map_to_other"
```

---

## 九、推荐枚举文件

```yaml
enums:
  owner_type:
    values: [human, llm, runtime, coordinator]

  capability_type:
    values: [execution, cognitive]

  relation_namespace:
    values: [concept_relation_types, essence_relation_types]

  concept_relation_type:
    values: [contains, depends_on, produces, validates, controls, references, blocks, resolves]

  essence_relation_type:
    values: [parallel, sequence, causal, depends_on, reinforces, conflicts_with, blocks, leads_to]

  rule_effect:
    values: [allow, deny, require_approval]

  gate_result:
    values: [pass, fail, require_review]

  severity:
    values: [blocker, major, minor, warning]

  confirmation_type:
    values: [machine_allowed, human_required, human_or_high_confidence_required]

  update_mode:
    values: [patch, minor, major]
```
