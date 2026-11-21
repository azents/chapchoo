import { useId } from "react";
import type { ReactElement } from "react";
import { NavigationIcon } from "./NavigationIcon";
import classes from "./SingleChoiceRows.module.css";

export function SingleChoiceRows({
  label,
  choices,
  value,
  disabled,
  onChange,
}: {
  label: string;
  choices: readonly { id: string; label: string }[];
  value: string | null;
  disabled?: boolean;
  onChange: (value: string) => void;
}): ReactElement {
  const name = useId();
  return (
    <fieldset className={classes.group} disabled={disabled}>
      <legend className={classes.legend}>{label}</legend>
      {choices.map((choice) => (
        <label className={classes.choice} key={choice.id}>
          <input
            type="radio"
            name={name}
            value={choice.id}
            checked={choice.id === value}
            onChange={() => onChange(choice.id)}
          />
          <span className={classes.label}>{choice.label}</span>
          <span className={classes.indicator} aria-hidden="true">
            {choice.id === value ? (
              <NavigationIcon name="check" size="sm" />
            ) : null}
          </span>
        </label>
      ))}
    </fieldset>
  );
}
