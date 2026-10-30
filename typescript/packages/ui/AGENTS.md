# Common web UI ownership

This package owns formal shared production-web presentation, expressed through
props and callbacks. Product apps and the independent
[`@chapchoo/design-reference`](../design-reference/README.md) may import
its explicit subpath exports. This package must not depend on either consumer.

For visual design context, read root [DESIGN.md](../../../DESIGN.md).

Change an existing shared component here and show the result from reference.
New experiments and mobile/native/non-web representations belong in reference
until actual shared-web reuse is selected. On promotion, move the implementation
here and update equivalent-role references to import it. A distinct, more general
reference representation may coexist with a web-specific common implementation;
document the relationship rather than treating it as an unnecessary copy.
Approved non-web representations stay in reference.
