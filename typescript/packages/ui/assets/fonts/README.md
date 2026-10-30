# Bundled web fonts

Pretendard v1.3.9 uses the upstream dynamic unicode-range subsets, preserving
all upstream ranges and variable weights. Browsers request only files needed
for the rendered text. No whole-font preload is used. The official subset set
is larger in aggregate than the full font; it reduces per-page transfer, not
the size of the complete installation archive.

Source commit: `orioncactus/pretendard@5c41199ea0024a9e0b2cb31735265056e5472d76`
([v1.3.9](https://github.com/orioncactus/pretendard/tree/v1.3.9/packages/pretendard/dist/web/variable)).
The replaced whole-font SHA-256 was
`9599f12fd42fc0bce1cd50b47a0c022e108d7aa64dd0d1bb0ed44f3282d900b4`,
verified identical to that release.

`pretendard-subsets.css` comes from `pretendardvariable-dynamic-subset.css`;
only the family alias (`Pretendard`) and relative directory (`pretendard/`)
were changed. All 92 WOFF2 files are unmodified upstream binaries. Keep the
complete set and its unicode ranges together when updating. The bundled
`Pretendard-LICENSE.txt` is the SIL Open Font License 1.1.

Geist Mono remains a separately loaded face for actual monospace content,
without a global preload. Its license is retained in `Geist-LICENSE.txt`.
