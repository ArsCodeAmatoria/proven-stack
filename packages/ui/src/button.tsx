"use client";

import type { ButtonHTMLAttributes, PropsWithChildren } from "react";

export type ButtonProps = PropsWithChildren<
  ButtonHTMLAttributes<HTMLButtonElement>
>;

/** Foundation primitive — no product business rules. */
export function Button({ children, ...props }: ButtonProps) {
  return (
    <button
      type="button"
      {...props}
      style={{
        padding: "0.5rem 1rem",
        borderRadius: "0.375rem",
        border: "1px solid #1f2937",
        background: "#111827",
        color: "#f9fafb",
        cursor: "pointer",
        ...((props.style as object) ?? {}),
      }}
    >
      {children}
    </button>
  );
}
