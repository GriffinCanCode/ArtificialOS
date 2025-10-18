**# Floating UI Module

Advanced positioning system for tooltips, popovers, dropdowns, and more using `@floating-ui/react`.

## Architecture

```
floating/
├── core/              # Pure functions and types
│   ├── types.ts       # TypeScript definitions
│   └── utils.ts       # Utility functions
├── hooks/             # React hooks
│   ├── useTooltip.ts  # Tooltip positioning
│   ├── usePopover.ts  # Popover interactions
│   ├── useDropdown.ts # Dropdown navigation
│   ├── useSelect.ts   # Select/combobox
│   ├── useContext.ts  # Context menu
│   └── useHover.ts    # Hover card
├── components/        # Reusable components
│   ├── Tooltip.tsx    # Smart tooltip
│   ├── Popover.tsx    # Interactive popover
│   ├── Dropdown.tsx   # Dropdown menu
│   ├── ContextMenu.tsx # Right-click menu
│   ├── Select.tsx     # Select/combobox
│   └── HoverCard.tsx  # Rich hover card
└── index.ts           # Public exports
```

## Features

### ✅ Smart Positioning
- Automatic overflow prevention
- Viewport-aware placement
- Collision detection
- Arrow indicators

### ✅ Rich Interactions
- Hover, click, focus triggers
- Keyboard navigation
- Type-ahead search
- Dismissal handling

### ✅ Accessibility
- ARIA attributes
- Screen reader support
- Keyboard navigation
- Focus management

### ✅ Customization
- Configurable delays
- Custom animations
- Flexible positioning
- Style theming

## Components

### Tooltip

Smart tooltip that never overflows the viewport.

```tsx
import { Tooltip } from "@/floating";

<Tooltip content="Hello world">
  <button>Hover me</button>
</Tooltip>
```

**Props:**
- `content`: Tooltip content
- `children`: Trigger element
- `delay`: Hover delay (default: 300ms)
- `position`: Positioning config
- `interaction`: Interaction config

**Features:**
- Smart positioning
- Hover + focus triggers
- Accessibility built-in
- Smooth animations

### Popover

Interactive popover with arrow and focus management.

```tsx
import { Popover } from "@/floating";

<Popover
  content={<div>Popover content</div>}
  arrow={true}
  modal={false}
>
  <button>Click me</button>
</Popover>
```

**Props:**
- `content`: Popover content
- `children`: Trigger element
- `arrow`: Show arrow (default: true)
- `modal`: Modal behavior (default: false)
- `position`: Positioning config
- `interaction`: Interaction config

**Features:**
- Click to open
- Arrow indicator
- Focus management
- Click-outside to close

### Dropdown

Dropdown menu with keyboard navigation.

```tsx
import { Dropdown } from "@/floating";

const items = [
  { value: "edit", label: "Edit", icon: <Edit /> },
  { value: "delete", label: "Delete", icon: <Trash /> },
];

<Dropdown items={items} onSelect={(value) => console.log(value)}>
  <button>Actions</button>
</Dropdown>
```

**Props:**
- `items`: Menu items
- `children`: Trigger element
- `onSelect`: Selection callback
- `position`: Positioning config
- `interaction`: Interaction config

**Features:**
- Keyboard navigation (↑↓)
- Focus management
- Item icons
- Dividers
- Disabled items

### ContextMenu

Right-click context menu.

```tsx
import { ContextMenu } from "@/floating";

const items = [
  { value: "copy", label: "Copy" },
  { divider: true },
  { value: "delete", label: "Delete" },
];

<ContextMenu items={items} onSelect={(value) => console.log(value)}>
  <div>Right-click me</div>
</ContextMenu>
```

**Props:**
- `items`: Menu items
- `children`: Trigger element
- `onSelect`: Selection callback

**Features:**
- Right-click trigger
- Smart positioning at cursor
- Keyboard navigation
- Auto-dismiss

### Select

Select/combobox with search.

```tsx
import { Select } from "@/floating";

const options = [
  { value: "1", label: "Option 1" },
  { value: "2", label: "Option 2" },
  { value: "3", label: "Option 3" },
];

<Select
  options={options}
  value={value}
  onChange={setValue}
  placeholder="Select option"
  searchable={true}
/>
```

**Props:**
- `options`: Select options
- `value`: Current value
- `onChange`: Change callback
- `placeholder`: Placeholder text
- `disabled`: Disabled state
- `searchable`: Enable search (default: false)

**Features:**
- Keyboard navigation
- Type-ahead search
- Check indicators
- Disabled options
- Groups support

### HoverCard

Rich hover card with delayed display.

```tsx
import { HoverCard } from "@/floating";

<HoverCard
  content={<div>Rich content here</div>}
  openDelay={700}
  closeDelay={300}
>
  <span>Hover me</span>
</HoverCard>
```

**Props:**
- `content`: Card content
- `children`: Trigger element
- `openDelay`: Open delay (default: 700ms)
- `closeDelay`: Close delay (default: 300ms)
- `position`: Positioning config

**Features:**
- Delayed display
- Rich content support
- Hover + focus triggers
- Smooth transitions

## Hooks

### useTooltip

```tsx
import { useTooltip } from "@/floating";

const tooltip = useTooltip({
  position: { placement: "top" },
  interaction: { delay: 300 },
});

return (
  <>
    <button {...tooltip.getReferenceProps()} ref={tooltip.refs.setReference}>
      Hover me
    </button>
    {tooltip.isOpen && (
      <div {...tooltip.getFloatingProps()} ref={tooltip.refs.setFloating}>
        Tooltip content
      </div>
    )}
  </>
);
```

### usePopover

```tsx
import { usePopover } from "@/floating";

const popover = usePopover({
  position: { placement: "bottom" },
  modal: false,
  arrow: true,
});
```

### useDropdown

```tsx
import { useDropdown } from "@/floating";

const dropdown = useDropdown({
  position: { placement: "bottom-start" },
  itemCount: items.length,
});
```

### useSelect

```tsx
import { useSelect } from "@/floating";

const select = useSelect({
  searchable: true,
});
```

### useContext

```tsx
import { useContext } from "@/floating";

const context = useContext();

// Open at cursor position
context.open(event.clientX, event.clientY);
```

### useHoverCard

```tsx
import { useHoverCard } from "@/floating";

const hoverCard = useHoverCard({
  openDelay: 700,
  closeDelay: 300,
});
```

## Configuration

### Position Config

```tsx
interface PositionConfig {
  placement?: Placement; // "top" | "bottom" | "left" | "right" | ...
  strategy?: Strategy;   // "absolute" | "fixed"
  middleware?: Middleware[];
  offset?: number;
}
```

**Placements:**
- `top`, `top-start`, `top-end`
- `bottom`, `bottom-start`, `bottom-end`
- `left`, `left-start`, `left-end`
- `right`, `right-start`, `right-end`

### Interaction Config

```tsx
interface InteractionConfig {
  trigger?: "hover" | "click" | "focus" | "manual";
  delay?: number | { open?: number; close?: number };
  closeOnClickOutside?: boolean;
  closeOnEscape?: boolean;
  closeOnScroll?: boolean;
}
```

### Accessibility Config

```tsx
interface AccessibilityConfig {
  role?: "tooltip" | "dialog" | "menu" | "listbox";
  describedBy?: boolean;
  labelledBy?: boolean;
}
```

## Styling

All components come with beautiful default styles following the project's design system:
- Glass morphism effects
- Purple accent colors
- Smooth animations
- Dark theme

### Custom Styles

Components use standard CSS classes that can be overridden:

```css
.tooltip { /* override tooltip styles */ }
.popover { /* override popover styles */ }
.dropdown { /* override dropdown styles */ }
```

## Use Cases

### Form Inputs
- Field validation tooltips
- Help popovers
- Select dropdowns
- Combobox search

### Navigation
- Action menus
- Context menus
- Navigation dropdowns
- User menus

### Data Display
- Info tooltips
- Rich hover cards
- Detail popovers
- Preview cards

### Interactions
- Confirmation dialogs
- Quick actions
- Keyboard shortcuts
- Status indicators

## Best Practices

### 1. Choose the Right Component
- **Tooltip**: Brief info, non-interactive
- **Popover**: Rich content, interactive
- **Dropdown**: Lists, actions
- **ContextMenu**: Right-click actions
- **Select**: Form inputs, options
- **HoverCard**: Rich previews

### 2. Accessibility First
- Always provide meaningful content
- Use semantic HTML
- Support keyboard navigation
- Test with screen readers

### 3. Performance
- Use `memo` for expensive content
- Lazy load heavy content
- Limit open popovers
- Clean up on unmount

### 4. User Experience
- Use appropriate delays
- Don't overflow content
- Provide clear triggers
- Handle edge cases

## Testing

Components include comprehensive test coverage:

```tsx
import { render, screen, userEvent } from "@testing-library/react";
import { Tooltip } from "@/floating";

it("shows tooltip on hover", async () => {
  render(
    <Tooltip content="Test tooltip">
      <button>Trigger</button>
    </Tooltip>
  );

  const trigger = screen.getByRole("button");
  await userEvent.hover(trigger);

  expect(screen.getByText("Test tooltip")).toBeInTheDocument();
});
```

## Migration from Native Elements

### Before: Title Attribute
```tsx
<button title="Click me">Button</button>
```

### After: Tooltip Component
```tsx
<Tooltip content="Click me">
  <button>Button</button>
</Tooltip>
```

### Before: Manual Positioning
```tsx
<div style={{ position: "absolute", top: y, left: x }}>
  Menu
</div>
```

### After: Dropdown Component
```tsx
<Dropdown items={items}>
  <button>Menu</button>
</Dropdown>
```

## New Capabilities Unlocked

### 🎯 Smart Positioning
- Automatic overflow prevention
- Viewport-aware placement
- Collision avoidance
- Dynamic repositioning

### ⌨️ Keyboard Navigation
- Arrow key support
- Tab navigation
- Type-ahead search
- Focus management

### ♿ Accessibility
- ARIA attributes
- Screen reader support
- Keyboard accessible
- Focus trapping

### 🎨 Rich Content
- HTML content
- React components
- Custom styling
- Animations

### 📱 Responsive
- Mobile-friendly
- Touch support
- Viewport adaptation
- Orientation handling

## Technical Note: ESLint Exception

This module has a **strategic ESLint exception** for `react-hooks/refs` in `/.eslintrc.cjs`.

### Why?
`@floating-ui/react` intentionally accesses refs during render for positioning calculations. This pattern:
- Is safe and battle-tested across millions of React apps
- Works correctly with React 18's concurrent rendering
- Predates React's stricter hooks rules

### The Alternative (Not Chosen)
Refactoring to eliminate ref access during render would require:
- Major changes across 6+ component types
- Risk of breaking production-tested positioning logic
- Significant engineering effort with minimal practical benefit

### Future Migration Path
When [CSS Anchor Positioning](https://developer.chrome.com/blog/anchor-positioning-api) achieves broad browser support (currently Chrome 125+), we'll migrate to native CSS positioning which:
- Eliminates JavaScript positioning entirely
- Provides better performance
- Removes this linting issue naturally

**Last reviewed:** 2025-10-17

## Examples

See `tests/floating/` for comprehensive examples and test cases.
