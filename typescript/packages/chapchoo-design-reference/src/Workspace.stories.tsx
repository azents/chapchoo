import { ReferenceFrame } from "./ReferenceFrame";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  title: "Start here/Workspace",
  component: ReferenceFrame,
  parameters: {
    docs: {
      description: {
        component:
          "An independent workspace for general design references across platforms. These initial stories explain ownership; they do not select Chapchoo's visual identity.",
      },
    },
  },
} satisfies Meta<typeof ReferenceFrame>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Purpose: Story = {
  args: {
    title: "Design reference workspace",
    note: "Chapchoo's visual design will be developed from scratch. The current Azents-derived styling is provisional.",
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
