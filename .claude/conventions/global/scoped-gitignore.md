---
title: "Place directory-owned ignore patterns in that subtree's .gitignore; reserve the root file for repository-wide or root-owned rules."
---

# Scope Git ignore rules to their owner

- Put generated or local artifact patterns in the nearest owning subtree's `.gitignore`, not a root entry naming that subtree.
- Keep genuinely repository-wide patterns and root-owned artifacts in the root file; share rules at the lowest common owning ancestor.
- Use directory-relative `/` anchors for outputs owned by that directory. When moving rules, verify nested matching and negations with `git check-ignore -v --no-index`, including tracked paths; use `git ls-files` to confirm tracked-file membership.

## Bad

```gitignore
# /.gitignore
typescript/apps/web/public/favicon.ico
```

## Good

```gitignore
# /typescript/apps/web/.gitignore
/public/favicon.ico
```
