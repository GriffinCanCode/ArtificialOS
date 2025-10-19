# CVA (Class Variance Authority) Setup

Type-safe component variant management for Tailwind CSS in the dynamic UI system.

## Overview

CVA provides type-safe component variants that ensure consistency across AI-generated and manually-created components. It integrates with the dynamic renderer architecture to handle variant application from backend-generated specifications.

## Architecture Integration

Component variants flow from backend specifications through the frontend type system:

```
UISpec JSON → DynamicRenderer → CVA variant functions → CSS classes
```

Backend components can specify variants as part of their props, and the frontend applies the correct CSS classes automatically through CVA.

## Implementation Location

Variant definitions are centralized in:
- `ui/src/core/utils/animation/componentVariants.ts` - All CVA variant definitions and type exports
- `ui/src/features/dynamics/styles/_variants.css` - Corresponding CSS class implementations

## Available Variants

### Button Variants

```typescript
import { buttonVariants, cn } from "../utils/animation/componentVariants";

<button className={cn(buttonVariants({ variant: "primary", size: "large" }))}>
  Click Me
</button>
```

**Variants:**
- `variant`: default, primary, secondary, danger, ghost, outline
- `size`: small, medium, large
- `fullWidth`: boolean

**Backend Usage:**
```python
button = Templates.button(
    id="submit-btn",
    text="Submit",
    on_click="form.submit",
    variant="primary"
)
```

### Input Variants

```typescript
<input className={cn(inputVariants({ variant: "filled", size: "large", error: true }))} />
```

**Variants:**
- `variant`: default, filled, outline, underline
- `size`: small, medium, large
- `error`: boolean
- `disabled`: boolean

### Text Variants

```typescript
<p className={cn(textVariants({ variant: "h1", weight: "bold", color: "accent" }))}>
  Heading
</p>
```

**Variants:**
- `variant`: h1, h2, h3, body, caption, label
- `weight`: normal, medium, semibold, bold
- `color`: primary, secondary, accent, muted, error, success
- `align`: left, center, right

### Container Variants

```typescript
<div className={cn(containerVariants({ 
  layout: "horizontal", 
  spacing: "large",
  align: "center",
  justify: "between"
}))}>
  {children}
</div>
```

**Variants:**
- `layout`: vertical, horizontal
- `spacing`: none, small, medium, large
- `padding`: none, small, medium, large
- `align`: start, center, end, stretch
- `justify`: start, center, end, between, around

### Grid Variants

```typescript
<div className={cn(gridVariants({ columns: 4, spacing: "large", responsive: true }))}>
  {items}
</div>
```

**Variants:**
- `columns`: 1, 2, 3, 4, 5, 6
- `spacing`: none, small, medium, large
- `responsive`: boolean

### Card Variants

```typescript
<div className={cn(cardVariants({ variant: "elevated", hoverable: true, interactive: true }))}>
  Card content
</div>
```

**Variants:**
- `variant`: default, elevated, outlined, ghost
- `padding`: none, small, medium, large
- `hoverable`: boolean
- `interactive`: boolean

## Usage in DynamicRenderer

The `DynamicRenderer` component applies CVA variants based on props from backend specifications. Component rendering automatically maps variant props to the appropriate CVA functions:

```typescript
case "button":
  return (
    <button
      className={cn(
        buttonVariants({
          variant: component.props?.variant as any,
          size: component.props?.size as any,
          fullWidth: component.props?.fullWidth,
        })
      )}
      onClick={() => handleEvent("click")}
    >
      {component.props?.text || "Button"}
    </button>
  );
```

## Backend Component Templates

Component templates in `ai-service/src/agents/ui_generator.py` provide a consistent interface for generating typed components:

```python
class Templates:
    @staticmethod
    def button(
        id: str, 
        text: str, 
        on_click: Optional[str] = None,
        variant: str = "default"
    ) -> BlueprintComponent:
        return BlueprintComponent(
            type="button",
            id=id,
            props={"text": text, "variant": variant},
            on_event={"click": on_click} if on_click else None
        )
```

## CSS Implementation

Variant classes are defined in `ui/src/features/dynamics/styles/_variants.css` and organized by component type. Each variant applies specific styling through Tailwind-defined custom CSS properties:

```css
.button-primary {
  background: var(--gradient-primary);
  border-color: var(--color-primary-500);
}

.button-primary:hover {
  background: var(--gradient-primary-hover);
  box-shadow: var(--shadow-glow-primary);
}
```

## Utility Functions

### `cn()` - Class Name Combiner

Combines multiple class sources, handling undefined and falsy values:

```typescript
import { cn } from "../utils/animation/componentVariants";

<div className={cn(
  "base-class",
  conditionalClass && "conditional-class",
  variantFunction({ variant: "primary" })
)}>
```

### `extractVariantProps()` - Props Separator

Separates variant props from other component props for selective application:

```typescript
import { extractVariantProps } from "../utils/animation/componentVariants";

const [variantProps, otherProps] = extractVariantProps(
  props,
  ["variant", "size", "fullWidth"]
);
```

## JSON Specification Example

Components can include variant specifications in their JSON representation:

```json
{
  "type": "button",
  "id": "submit-btn",
  "props": {
    "text": "Submit",
    "variant": "primary",
    "size": "large",
    "fullWidth": true
  },
  "on_event": {"click": "form.submit"}
}
```

The `DynamicRenderer` applies the correct CVA variant classes based on these props.

## Type Safety

CVA provides compile-time type inference for all variants:

```typescript
type ButtonVariants = VariantProps<typeof buttonVariants>;
// { variant?: "default" | "primary" | ..., size?: "small" | "medium" | "large", ... }
```

This ensures invalid variant values are caught during development.

## Adding New Variants

1. **Define CVA Variants in TypeScript:**
```typescript
// ui/src/core/utils/animation/componentVariants.ts
export const newComponentVariants = cva("new-component-base", {
  variants: {
    variant: {
      default: "new-variant-default",
      special: "new-variant-special",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});
```

2. **Add CSS Classes:**
```css
/* ui/src/features/dynamics/styles/_variants.css */
.new-variant-default { /* styles */ }
.new-variant-special { /* styles */ }
```

3. **Update Backend Template (Optional):**
```python
# ai-service/src/agents/ui_generator.py
@staticmethod
def new_component(id: str, variant: str = "default") -> BlueprintComponent:
    return BlueprintComponent(
        type="new_component",
        id=id,
        props={"variant": variant}
    )
```

4. **Update DynamicRenderer:**
```typescript
case "new_component":
  return (
    <div className={cn(newComponentVariants({ 
      variant: component.props?.variant 
    }))}>
      {/* component content */}
    </div>
  );
```

## Benefits

- **Type Safety** - Invalid variants caught at compile time
- **Consistency** - Unified styling system across all components
- **Maintainability** - Centralized variant definitions
- **Developer Experience** - Auto-complete for all variant options
- **Performance** - No runtime overhead, just static class strings
- **Backend Integration** - Straightforward for LLM to generate valid variant props

## References

- [CVA Documentation](https://cva.style/docs)
- [Tailwind CSS](https://tailwindcss.com)
- Component Variants: `ui/src/core/utils/animation/componentVariants.ts`
- Backend Templates: `ai-service/src/agents/ui_generator.py`

