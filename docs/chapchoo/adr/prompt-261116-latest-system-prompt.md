---
title: "Latest System Prompt Replacement Authority"
document_role: primary
document_type: adr
snapshot_id: prompt-261116
created: 2026-11-16
tags: [agent, instructions, persistence]
---

# Latest System Prompt Replacement Authority

Authority: [prompt-261116/REQ](../requirements/prompt-261116-latest-system-prompt.md).

## Fixed outcome map

The requester directly selected latest-only retention, removal of historical
access and separate-page navigation. These supersede the historical retention
contract in the implemented `prompt-261115` snapshot. There is no remaining
material choice between retaining and deleting prompt history.

The existing operational preparation identity, exact execution checks and native
delivery ownership remain authoritative. A timestamp-selected record cannot
replace those checks. The implementation derives a single current payload from
that existing selection, removes obsolete payloads transactionally and enforces
one payload per Agent. Current saved owner text remains a separate overwrite-only
setting. No additional mode, settings revision or compatibility path is introduced.

Identifiers, helper boundaries and page composition are local implementation
details. No new material ADR decision is required.
