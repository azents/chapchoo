import { WorkspaceChrome } from "@chapchoo/ui/WorkspaceChrome";
import { AgentNavigation } from "./AgentNavigation";
import type { NavigationModel } from "./useNavigation";
import type { ReactElement } from "react";

export function NavigationChrome(props: NavigationModel): ReactElement {
  return (
    <WorkspaceChrome
      {...props}
      navigationContent={
        props.agentNavigation ? (
          <AgentNavigation {...props.agentNavigation} />
        ) : (
          void 0
        )
      }
    />
  );
}
