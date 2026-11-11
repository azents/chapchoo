# Product-design HTML archive

A small curated set of frozen visual records, not maintained UI or current-system
requirements. Read [AGENTS.md](AGENTS.md) and the `product-design-snapshot` Skill.
New concepts start by copying the current UI, then change only the requested parts,
iterate with feedback, receive explicit confirmation and produce an independent dump.

## Confirmed Chat records

[Chat record inspection](chat-records/README.md) retains the confirmed
2026-11-12 KST one-line record list and expanded-command direction as two
independent real-component DOM/CSS dumps. Open the [mobile list](chat-records/mobile-records.html)
or [desktop command](chat-records/desktop-command.html). Their fictional data,
static-control and viewport limits are recorded in the collection index.

## Confirmed harness setup

[Harness setup](harness-setup/README.md) is the requester-confirmed integrated
concept from 2026-11-06 KST: eight core states, local shared CSS and deduplicated
assets (approximately 1.05 MB). Its approval and fidelity limits are recorded in
that index. It supersedes the older unconfirmed authentication proposals below,
without retroactively changing their historical status.

## Core roadmap

[Agent roadmap](agent-roadmap/index.html) retains 15 representative state pages:
Agent list/profile, the scoped three-step Codex creation flow, one connection
entry, team selection/creation and representative cloud/power management.
Five principal screens have desktop/mobile pairs; supporting screens use one
representative desktop state. Intermediate drafts, competing alternatives and
exhaustive error/responsive permutations are intentionally omitted.

Imported 2026-11-05 KST. These are historical **Concept mocks**, not newly authored
component renders. The scoped Codex additions remain unconfirmed; archiving does
not grant design approval, implementation authority, merge or deployment permission.

The core HTML and 20 deduplicated assets total approximately **0.83 MB** before
indexes/licenses. Only needed font subsets are retained. Verification PNGs,
authoring JavaScript/React, full source bundles and private research are not included.

## Open locally

From the repository root:

```sh
python -m http.server 6009 --bind 127.0.0.1 --directory typescript/packages/design-reference/snapshots
```

Open `http://127.0.0.1:6009/`. No build, Storybook, OpenDesign, application bundle
or backend is needed. Stop the server after review. Controls are illustrative;
they do not authenticate, save, provision Computers or execute Agent work.

See the [core index](agent-roadmap/README.md) for status and fidelity limitations,
and [asset provenance and licenses](ASSETS.md).
