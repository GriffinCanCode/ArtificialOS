/**
 * Component Variant Definitions
 * Type-safe variant management using CVA for dynamic components
 */

import { cva, type VariantProps } from "class-variance-authority";

// ============================================================================
// Button Variants
// ============================================================================

export const buttonVariants = cva("dynamic-button", {
  variants: {
    variant: {
      default: "button-default",
      primary: "button-primary",
      secondary: "button-secondary",
      danger: "button-danger",
      ghost: "button-ghost",
      outline: "button-outline",
    },
    size: {
      small: "button-sm",
      medium: "button-md",
      large: "button-lg",
    },
    fullWidth: {
      true: "button-full-width",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "medium",
  },
});

export type ButtonVariants = VariantProps<typeof buttonVariants>;

// ============================================================================
// Input Variants
// ============================================================================

export const inputVariants = cva("dynamic-input", {
  variants: {
    variant: {
      default: "input-default",
      filled: "input-filled",
      outline: "input-outline",
      underline: "input-underline",
    },
    size: {
      small: "input-sm",
      medium: "input-md",
      large: "input-lg",
    },
    error: {
      true: "input-error",
    },
    disabled: {
      true: "input-disabled",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "medium",
  },
});

export type InputVariants = VariantProps<typeof inputVariants>;

// ============================================================================
// Text Variants
// ============================================================================

export const textVariants = cva("dynamic-text", {
  variants: {
    variant: {
      h1: "text-h1",
      h2: "text-h2",
      h3: "text-h3",
      body: "text-body",
      caption: "text-caption",
      label: "text-label",
    },
    weight: {
      normal: "text-weight-normal",
      medium: "text-weight-medium",
      semibold: "text-weight-semibold",
      bold: "text-weight-bold",
    },
    color: {
      primary: "text-color-primary",
      secondary: "text-color-secondary",
      accent: "text-color-accent",
      muted: "text-color-muted",
      error: "text-color-error",
      success: "text-color-success",
    },
    align: {
      left: "text-align-left",
      center: "text-align-center",
      right: "text-align-right",
    },
  },
  defaultVariants: {
    variant: "body",
    weight: "normal",
    color: "primary",
    align: "left",
  },
});

export type TextVariants = VariantProps<typeof textVariants>;

// ============================================================================
// Container Variants
// ============================================================================

export const containerVariants = cva("dynamic-container", {
  variants: {
    layout: {
      vertical: "container-vertical",
      horizontal: "container-horizontal",
    },
    spacing: {
      none: "container-spacing-none",
      small: "container-spacing-sm",
      medium: "container-spacing-md",
      large: "container-spacing-lg",
    },
    padding: {
      none: "container-padding-none",
      small: "container-padding-sm",
      medium: "container-padding-md",
      large: "container-padding-lg",
    },
    align: {
      start: "container-align-start",
      center: "container-align-center",
      end: "container-align-end",
      stretch: "container-align-stretch",
    },
    justify: {
      start: "container-justify-start",
      center: "container-justify-center",
      end: "container-justify-end",
      between: "container-justify-between",
      around: "container-justify-around",
    },
  },
  defaultVariants: {
    layout: "vertical",
    spacing: "medium",
    padding: "none",
    align: "stretch",
    justify: "start",
  },
});

export type ContainerVariants = VariantProps<typeof containerVariants>;

// ============================================================================
// Grid Variants
// ============================================================================

export const gridVariants = cva("dynamic-grid", {
  variants: {
    columns: {
      1: "grid-cols-1",
      2: "grid-cols-2",
      3: "grid-cols-3",
      4: "grid-cols-4",
      5: "grid-cols-5",
      6: "grid-cols-6",
    },
    spacing: {
      none: "grid-spacing-none",
      small: "grid-spacing-sm",
      medium: "grid-spacing-md",
      large: "grid-spacing-lg",
    },
    responsive: {
      true: "grid-responsive",
    },
  },
  defaultVariants: {
    columns: 3,
    spacing: "medium",
  },
});

export type GridVariants = VariantProps<typeof gridVariants>;

// ============================================================================
// Card Variants (for Launcher and other cards)
// ============================================================================

export const cardVariants = cva("card-base", {
  variants: {
    variant: {
      default: "card-default",
      elevated: "card-elevated",
      outlined: "card-outlined",
      ghost: "card-ghost",
    },
    padding: {
      none: "card-padding-none",
      small: "card-padding-sm",
      medium: "card-padding-md",
      large: "card-padding-lg",
    },
    hoverable: {
      true: "card-hoverable",
    },
    interactive: {
      true: "card-interactive",
    },
  },
  defaultVariants: {
    variant: "default",
    padding: "medium",
  },
});

export type CardVariants = VariantProps<typeof cardVariants>;

// ============================================================================
// Category Button Variants (for Launcher)
// ============================================================================

export const categoryButtonVariants = cva("category-btn-base", {
  variants: {
    active: {
      true: "category-btn-active",
      false: "category-btn-inactive",
    },
    size: {
      small: "category-btn-sm",
      medium: "category-btn-md",
      large: "category-btn-lg",
    },
  },
  defaultVariants: {
    active: false,
    size: "medium",
  },
});

export type CategoryButtonVariants = VariantProps<typeof categoryButtonVariants>;

// ============================================================================
// Control Button Variants (for TitleBar)
// ============================================================================

export const controlButtonVariants = cva("control-btn-base", {
  variants: {
    type: {
      minimize: "control-btn-minimize",
      maximize: "control-btn-maximize",
      close: "control-btn-close",
    },
    state: {
      normal: "control-btn-normal",
      hover: "control-btn-hover",
      active: "control-btn-active",
    },
  },
  defaultVariants: {
    state: "normal",
  },
});

export type ControlButtonVariants = VariantProps<typeof controlButtonVariants>;

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Combines class names with proper precedence
 */
export function cn(...inputs: (string | undefined | null | false)[]): string {
  return inputs.filter(Boolean).join(" ");
}

/**
 * Extract variant props from component props
 * Useful for separating CVA variant props from other props
 */
export function extractVariantProps<T extends Record<string, any>>(
  props: T,
  validKeys: (keyof T)[]
): [Partial<T>, Partial<T>] {
  const variantProps: Partial<T> = {};
  const otherProps: Partial<T> = {};

  Object.entries(props).forEach(([key, value]) => {
    if (validKeys.includes(key as keyof T)) {
      variantProps[key as keyof T] = value;
    } else {
      otherProps[key as keyof T] = value;
    }
  });

  return [variantProps, otherProps];
}

