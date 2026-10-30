import { ReferenceFrame } from "./ReferenceFrame";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  title: "Start here/Workspace",
  component: ReferenceFrame,
  parameters: {
    docs: {
      description: {
        component:
          "Independent cross-platform design references. Read root DESIGN.md for visual context and the Chapchoo stories for the selected tone. This page explains workspace ownership.",
      },
    },
  },
} satisfies Meta<typeof ReferenceFrame>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Purpose: Story = {
  args: {
    title: "Design reference workspace",
    note: "Design principles live in root DESIGN.md. The Chapchoo catalog and shared web components use the selected tone.",
    children: (
      <>
        <section>
          <h2>Author and review design intent</h2>
          <p>
            Use static examples to compare composition, content hierarchy and
            cross-platform ideas. Product behavior tests and app UI review stay
            in the app workspaces.
          </p>
        </section>
        <section>
          <h2>Choose the implementation owner</h2>
          <ul>
            <li>Existing shared web component: import and modify common UI.</li>
            <li>
              New experiment: define it locally until selected for web reuse.
            </li>
            <li>
              Non-web representation: keep it here, even after design approval.
            </li>
          </ul>
        </section>
        <section>
          <h2>Promote only for shared web use</h2>
          <p>
            Move shared-web implementation responsibility to common UI and
            import it for equivalent roles. A distinct, more general reference
            may remain alongside a web-specific implementation. Document the
            relationship. Design approval alone is not promotion.
          </p>
        </section>
      </>
    ),
  },
};
