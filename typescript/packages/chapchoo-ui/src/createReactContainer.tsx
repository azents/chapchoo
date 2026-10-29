"use client";

import type { ComponentType, ReactElement } from "react";

/** Connect an app-owned state hook to a pure presentation component. */
export function createReactContainer<Input, Output extends object>(
  displayName: string,
  containerHook: (props: Input) => Output,
  defaultComponent: ComponentType<Output>,
): ComponentType<Input & { component?: ComponentType<Output> }>;
export function createReactContainer<Input, Output extends object>(
  displayName: string,
  containerHook: (props: Input) => Output,
): ComponentType<Input & { component: ComponentType<Output> }>;
export function createReactContainer<Input, Output extends object>(
  displayName: string,
  containerHook: (props: Input) => Output,
  defaultComponent?: ComponentType<Output>,
): ComponentType<Input & { component?: ComponentType<Output> }> {
  const useContainerHook = containerHook;
  function Container(
    props: Input & { component?: ComponentType<Output> },
  ): ReactElement | null {
    const Component = props.component ?? defaultComponent;
    const output = useContainerHook(props);
    if (Component === undefined) {
      return null;
    }
    return <Component {...output} />;
  }
  Container.displayName = displayName;
  return Container;
}
