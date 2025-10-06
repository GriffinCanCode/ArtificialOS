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
      vertical: "dynamic-container-vertical",
      horizontal: "dynamic-container-horizontal",
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
// Select/Dropdown Variants
// ============================================================================

export const selectVariants = cva("dynamic-select", {
  variants: {
    variant: {
      default: "select-default",
      filled: "select-filled",
      outline: "select-outline",
    },
    size: {
      small: "select-sm",
      medium: "select-md",
      large: "select-lg",
    },
    error: {
      true: "select-error",
    },
    disabled: {
      true: "select-disabled",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "medium",
  },
});

export type SelectVariants = VariantProps<typeof selectVariants>;

// ============================================================================
// Checkbox Variants
// ============================================================================

export const checkboxVariants = cva("dynamic-checkbox", {
  variants: {
    size: {
      small: "checkbox-sm",
      medium: "checkbox-md",
      large: "checkbox-lg",
    },
    variant: {
      default: "checkbox-default",
      primary: "checkbox-primary",
    },
    disabled: {
      true: "checkbox-disabled",
    },
  },
  defaultVariants: {
    size: "medium",
    variant: "default",
  },
});

export type CheckboxVariants = VariantProps<typeof checkboxVariants>;

// ============================================================================
// Radio Variants
// ============================================================================

export const radioVariants = cva("dynamic-radio", {
  variants: {
    size: {
      small: "radio-sm",
      medium: "radio-md",
      large: "radio-lg",
    },
    disabled: {
      true: "radio-disabled",
    },
  },
  defaultVariants: {
    size: "medium",
  },
});

export type RadioVariants = VariantProps<typeof radioVariants>;

// ============================================================================
// Textarea Variants
// ============================================================================

export const textareaVariants = cva("dynamic-textarea", {
  variants: {
    variant: {
      default: "textarea-default",
      filled: "textarea-filled",
      outline: "textarea-outline",
    },
    size: {
      small: "textarea-sm",
      medium: "textarea-md",
      large: "textarea-lg",
    },
    error: {
      true: "textarea-error",
    },
    disabled: {
      true: "textarea-disabled",
    },
    resize: {
      none: "textarea-resize-none",
      vertical: "textarea-resize-vertical",
      horizontal: "textarea-resize-horizontal",
      both: "textarea-resize-both",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "medium",
    resize: "vertical",
  },
});

export type TextareaVariants = VariantProps<typeof textareaVariants>;

// ============================================================================
// Image Variants
// ============================================================================

export const imageVariants = cva("dynamic-image", {
  variants: {
    fit: {
      cover: "image-fit-cover",
      contain: "image-fit-contain",
      fill: "image-fit-fill",
      none: "image-fit-none",
    },
    rounded: {
      none: "image-rounded-none",
      small: "image-rounded-sm",
      medium: "image-rounded-md",
      large: "image-rounded-lg",
      full: "image-rounded-full",
    },
  },
  defaultVariants: {
    fit: "cover",
    rounded: "none",
  },
});

export type ImageVariants = VariantProps<typeof imageVariants>;

// ============================================================================
// Slider Variants
// ============================================================================

export const sliderVariants = cva("dynamic-slider", {
  variants: {
    size: {
      small: "slider-sm",
      medium: "slider-md",
      large: "slider-lg",
    },
    variant: {
      default: "slider-default",
      primary: "slider-primary",
    },
    disabled: {
      true: "slider-disabled",
    },
  },
  defaultVariants: {
    size: "medium",
    variant: "default",
  },
});

export type SliderVariants = VariantProps<typeof sliderVariants>;

// ============================================================================
// Progress Variants
// ============================================================================

export const progressVariants = cva("dynamic-progress", {
  variants: {
    variant: {
      default: "progress-default",
      primary: "progress-primary",
      success: "progress-success",
      error: "progress-error",
    },
    size: {
      small: "progress-sm",
      medium: "progress-md",
      large: "progress-lg",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "medium",
  },
});

export type ProgressVariants = VariantProps<typeof progressVariants>;

// ============================================================================
// Badge Variants
// ============================================================================

export const badgeVariants = cva("dynamic-badge", {
  variants: {
    variant: {
      default: "badge-default",
      primary: "badge-primary",
      secondary: "badge-secondary",
      success: "badge-success",
      error: "badge-error",
      warning: "badge-warning",
    },
    size: {
      small: "badge-sm",
      medium: "badge-md",
      large: "badge-lg",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "medium",
  },
});

export type BadgeVariants = VariantProps<typeof badgeVariants>;

// ============================================================================
// Divider Variants
// ============================================================================

export const dividerVariants = cva("dynamic-divider", {
  variants: {
    orientation: {
      horizontal: "divider-horizontal",
      vertical: "divider-vertical",
    },
    variant: {
      solid: "divider-solid",
      dashed: "divider-dashed",
      dotted: "divider-dotted",
    },
  },
  defaultVariants: {
    orientation: "horizontal",
    variant: "solid",
  },
});

export type DividerVariants = VariantProps<typeof dividerVariants>;

// ============================================================================
// Tab Variants
// ============================================================================

export const tabVariants = cva("dynamic-tab", {
  variants: {
    variant: {
      default: "tab-default",
      bordered: "tab-bordered",
      pills: "tab-pills",
    },
    size: {
      small: "tab-sm",
      medium: "tab-md",
      large: "tab-lg",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "medium",
  },
});

export type TabVariants = VariantProps<typeof tabVariants>;

// ============================================================================
// Modal Variants
// ============================================================================

export const modalVariants = cva("dynamic-modal", {
  variants: {
    size: {
      small: "modal-sm",
      medium: "modal-md",
      large: "modal-lg",
      fullscreen: "modal-fullscreen",
    },
    centered: {
      true: "modal-centered",
    },
  },
  defaultVariants: {
    size: "medium",
    centered: true,
  },
});

export type ModalVariants = VariantProps<typeof modalVariants>;

// ============================================================================
// List Variants
// ============================================================================

export const listVariants = cva("dynamic-list", {
  variants: {
    variant: {
      default: "list-default",
      bordered: "list-bordered",
      striped: "list-striped",
    },
    spacing: {
      none: "list-spacing-none",
      small: "list-spacing-sm",
      medium: "list-spacing-md",
      large: "list-spacing-lg",
    },
  },
  defaultVariants: {
    variant: "default",
    spacing: "medium",
  },
});

export type ListVariants = VariantProps<typeof listVariants>;

// ============================================================================
// Canvas Variants
// ============================================================================

export const canvasVariants = cva("dynamic-canvas", {
  variants: {
    bordered: {
      true: "canvas-bordered",
    },
  },
  defaultVariants: {},
});

export type CanvasVariants = VariantProps<typeof canvasVariants>;

// ============================================================================
// Iframe Variants
// ============================================================================

export const iframeVariants = cva("dynamic-iframe", {
  variants: {
    bordered: {
      true: "iframe-bordered",
    },
    rounded: {
      true: "iframe-rounded",
    },
  },
  defaultVariants: {},
});

export type IframeVariants = VariantProps<typeof iframeVariants>;

// ============================================================================
// Video Variants
// ============================================================================

export const videoVariants = cva("dynamic-video", {
  variants: {
    fit: {
      cover: "video-fit-cover",
      contain: "video-fit-contain",
      fill: "video-fit-fill",
    },
    rounded: {
      none: "video-rounded-none",
      small: "video-rounded-sm",
      medium: "video-rounded-md",
      large: "video-rounded-lg",
    },
  },
  defaultVariants: {
    fit: "contain",
    rounded: "none",
  },
});

export type VideoVariants = VariantProps<typeof videoVariants>;

// ============================================================================
// Audio Variants
// ============================================================================

export const audioVariants = cva("dynamic-audio", {
  variants: {
    variant: {
      default: "audio-default",
      minimal: "audio-minimal",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

export type AudioVariants = VariantProps<typeof audioVariants>;

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
