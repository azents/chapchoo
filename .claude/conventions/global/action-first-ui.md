---
title: "When designing or changing UI, express constraints and recovery through state-aware controls and flows instead of explanatory labels."
---

# Action-first UI

- Prefer enabled, disabled, selected and pending states, contextual actions and preserved input over prose that explains how the UI should work.
- Avoid labels describing exceptions, excluded scope or internal guarantees when the interaction can enforce the rule and offer the next action.
- Keep essential field labels, accessible names, concise statuses, actionable errors and material cost or destructive-impact disclosures. A disabled control alone is not recovery: expose a reachable next action.
- Apply to product UI and wireframes, including General, Admin and shared design references.

## Bad

Keep chat enabled and add “Conversation is unavailable while the Computer is off.”

## Good

While the Computer is off, disable the composer and expose “Turn on Computer”; restore input after startup succeeds and offer retry on failure.
