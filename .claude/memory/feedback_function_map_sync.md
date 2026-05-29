---
name: function-map-sync
description: Code changes must be verified against function-map.yml — check before and after every modification
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 888ba448-657c-4112-b66b-a94332060795
---

# Function Map Sync Rule

When modifying code, always check `function-map.yml` first to verify the change aligns with the defined function map.

**Why:** function-map.yml is the authoritative function graph for the requirements module. Code that diverges from it creates drift between design intent and implementation, making the map useless as a navigation and dependency reference.

**How to apply:**
1. After every user message, determine if code will be modified
2. If code changes are needed, read `function-map.yml` to check: do the target files, actions, data contexts, and transitions match?
3. If match → proceed
4. If mismatch → evaluate: is this a reasonable extension? If yes, update function-map.yml along with the code. If no, present the concern and alternatives.
5. After code changes complete, verify consistency one more time
