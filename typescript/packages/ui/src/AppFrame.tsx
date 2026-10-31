import { AppHeader } from "./AppHeader";
import classes from "./AppFrame.module.css";
import type { AppHeaderProps } from "./AppHeader";
import type { ReactElement, ReactNode } from "react";

export interface AppFrameProps extends AppHeaderProps {
  children: ReactNode;
}

/** Let content fill the viewport while the header can grow with enlarged text. */
export function AppFrame({ children, ...header }: AppFrameProps): ReactElement {
  return (
    <div className={classes.frame}>
      <AppHeader {...header} />
      {children}
    </div>
  );
}
