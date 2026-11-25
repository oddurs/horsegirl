"use client";

import { useState } from "react";
import styles from "./components.module.css";

export type Option = { value: string; label: string };

// A row of toggle buttons where exactly one is pressed, as the lens picker on
// the concept page.
export function SegmentedControl({
  label,
  options,
  defaultValue,
  onChange,
}: {
  label: string;
  options: Option[];
  defaultValue: string;
  onChange?: (value: string) => void;
}) {
  const [value, setValue] = useState(defaultValue);
  return (
    <fieldset className={styles.seg} aria-label={label}>
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          aria-pressed={option.value === value}
          onClick={() => {
            setValue(option.value);
            onChange?.(option.value);
          }}
        >
          {option.label}
        </button>
      ))}
    </fieldset>
  );
}
