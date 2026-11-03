---
title: "Review every UI translation in its language and interaction context; express the intended action naturally instead of copying English wording."
---

# Contextual UI localization

- Review each changed label, sentence and state individually in every supported locale, in its rendered UI context: audience, destination/action, scope, space and tone. English is semantic input, not a wording template.
- Write idiomatic local product language. An administrator-page entry in Korean is `관리자 페이지`, not the ambiguous action `관리`; `에이전트 만들기` is preferable to mixed-language `Agent 만들기` in ordinary navigation.
- Retain product names and precise technical terms such as Kubernetes, Docker, Pod, PVC, CIDR and CPU when local wording would be less familiar or accurate. Capitalization in English alone is not a reason to retain English. Keep terminology consistent within each locale.
- Preserve outcome, permission and recovery meaning. Explain internal field paths, enum values and reason codes in user language rather than exposing them as primary copy.
- Keep keys aligned, but evaluate naturalness, truncation/wrapping and contextual ambiguity per locale before completion. Do not treat key parity or English readability as translation review.
