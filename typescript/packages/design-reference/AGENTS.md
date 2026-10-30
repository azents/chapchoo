# Design reference ownership

Read [README.md](README.md) before authoring references.
Read root `DESIGN.md` when present and follow
[DESIGN_AUTHORING.md](DESIGN_AUTHORING.md) for visual context, token authority,
platform differences, mobile density and accessible review.

This package owns general-purpose cross-platform design references, local
experiments and web representations of non-web UI. App functional testing and
app UI review remain in the app workspaces.

Current Azents-derived styling is provisional, not an approved Chapchoo design
system. Initial examples are structural scaffolding. Do not turn existing style
values into approved tokens or extend their catalog without a design request.

Import existing formal web components from `@chapchoo/ui`; modify them in their
owning package. Create reference-local components for new experiments and non-web
representations. Design approval alone is not promotion. Move an implementation
to common UI only when selected for shared web use. Import common UI for an
equivalent design role. A distinct, more general representation may remain here
when common UI is web-specific; document their relationship and intentional
differences. Remove unnecessary same-role copies. Approved non-web
representations remain here.

Use static fixtures and small explanatory interactions, with no app imports,
backend transport or login. Keep providers scoped to the design they represent;
the provisional common-UI theme is not the workspace-wide design authority.

Run root frontend quality checks, the boundary check and this Storybook build.
Visually check the relevant rendered stories; a successful build is not visual
or native-platform verification.
