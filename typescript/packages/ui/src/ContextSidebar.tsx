import { InteractiveSurface } from "./InteractiveSurface";
import { NavigationIcon } from "./NavigationIcon";
import classes from "./ContextSidebar.module.css";
import type { NavigationItem } from "./AppNavigation";
import type { ReactElement, ReactNode } from "react";

/** The Chat sidebar composition is shared by all context pages and Drawers. */
export function ContextSidebar(props: {
  label: string;
  header?: ReactNode;
  content?: ReactNode;
  items: readonly NavigationItem[];
  onSelect?: () => void;
}): ReactElement {
  return (
    <div className={classes.navigation}>
      {props.header ? (
        <div className={classes.header}>{props.header}</div>
      ) : null}
      {props.content}
      <nav
        aria-label={props.label}
        className={classes.links}
        data-footer={Boolean(props.content) || void 0}
      >
        {props.items.map((item) => (
          <InteractiveSurface
            key={item.key}
            {...(item.href ? { href: item.href } : {})}
            className={classes.link}
            aria-current={item.active ? "page" : void 0}
            data-selected={item.active || void 0}
            onClick={() => {
              item.onSelect?.();
              props.onSelect?.();
            }}
          >
            {item.icon ? <NavigationIcon name={item.icon} /> : null}
            <span>{item.label}</span>
          </InteractiveSurface>
        ))}
      </nav>
    </div>
  );
}
