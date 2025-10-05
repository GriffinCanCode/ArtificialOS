/**
 * Component Registration
 * Registers all component renderers with the registry
 * Import this once at app initialization
 */

import { registry } from "../core/registry";
import type { ComponentRenderer } from "../core/types";

// Import all component renderers
import { Button } from "../components/primitives/Button";
import { Input } from "../components/primitives/Input";
import { Text } from "../components/primitives/Text";
import { Checkbox } from "../components/primitives/Checkbox";
import { Radio } from "../components/primitives/Radio";
import { Slider } from "../components/primitives/Slider";

import { Container } from "../components/layout/Container";
import { Grid } from "../components/layout/Grid";
import { List } from "../components/layout/List";

import { Select } from "../components/forms/Select";
import { Textarea } from "../components/forms/Textarea";

import { Image } from "../components/media/Image";
import { Video } from "../components/media/Video";
import { Audio } from "../components/media/Audio";
import { Canvas } from "../components/media/Canvas";

import { Badge } from "../components/ui/Badge";
import { Card } from "../components/ui/Card";
import { Divider } from "../components/ui/Divider";
import { Modal } from "../components/ui/Modal";
import { Tabs } from "../components/ui/Tabs";

import { AppShortcut } from "../components/special/AppShortcut";
import { Iframe } from "../components/special/Iframe";
import { Progress } from "../components/special/Progress";

// Import validation schemas
import {
  buttonSchema,
  inputSchema,
  textSchema,
  checkboxSchema,
  radioSchema,
  sliderSchema,
} from "../schemas/primitives";

import { containerSchema, gridSchema, listSchema } from "../schemas/layout";

import { selectSchema, textareaSchema } from "../schemas/forms";

import { imageSchema, videoSchema, audioSchema, canvasSchema } from "../schemas/media";

import { badgeSchema, cardSchema, dividerSchema, modalSchema, tabsSchema } from "../schemas/ui";

import { appShortcutSchema, iframeSchema, progressSchema } from "../schemas/special";

// ============================================================================
// Component Registry Definitions
// ============================================================================

const componentRenderers: ComponentRenderer[] = [
  // Primitives
  { type: "button", render: Button, schema: buttonSchema, category: "primitive" },
  { type: "input", render: Input, schema: inputSchema, category: "primitive" },
  { type: "text", render: Text, schema: textSchema, category: "primitive" },
  { type: "checkbox", render: Checkbox, schema: checkboxSchema, category: "primitive" },
  { type: "radio", render: Radio, schema: radioSchema, category: "primitive" },
  { type: "slider", render: Slider, schema: sliderSchema, category: "primitive" },

  // Layout
  { type: "container", render: Container, schema: containerSchema, category: "layout" },
  { type: "grid", render: Grid, schema: gridSchema, category: "layout" },
  { type: "list", render: List, schema: listSchema, category: "layout" },

  // Forms
  { type: "select", render: Select, schema: selectSchema, category: "form" },
  { type: "textarea", render: Textarea, schema: textareaSchema, category: "form" },

  // Media
  { type: "image", render: Image, schema: imageSchema, category: "media" },
  { type: "video", render: Video, schema: videoSchema, category: "media" },
  { type: "audio", render: Audio, schema: audioSchema, category: "media" },
  { type: "canvas", render: Canvas, schema: canvasSchema, category: "media" },

  // UI
  { type: "badge", render: Badge, schema: badgeSchema, category: "ui" },
  { type: "card", render: Card, schema: cardSchema, category: "ui" },
  { type: "divider", render: Divider, schema: dividerSchema, category: "ui" },
  { type: "modal", render: Modal, schema: modalSchema, category: "ui" },
  { type: "tabs", render: Tabs, schema: tabsSchema, category: "ui" },

  // Special
  { type: "app-shortcut", render: AppShortcut, schema: appShortcutSchema, category: "special" },
  { type: "iframe", render: Iframe, schema: iframeSchema, category: "special" },
  { type: "progress", render: Progress, schema: progressSchema, category: "special" },
];

// ============================================================================
// Registration Function
// ============================================================================

/**
 * Register all component renderers
 * Call this once during app initialization
 */
export function registerAllComponents(): void {
  registry.registerAll(componentRenderers);

  if (process.env.NODE_ENV === "development") {
    const stats = registry.getStats();
    console.log("📦 Component Registry Initialized:", stats);
  }
}

// Auto-register on import (side effect)
registerAllComponents();
