# CVA (Class Variance Authority) Setup

Complete type-safe variant management for Tailwind CSS in our dynamic UI system.

## Overview

CVA provides type-safe component variants that ensure consistency across AI-generated and manually-created components. It integrates perfectly with our dynamic renderer architecture.

## Architecture Integration

### Frontend Flow
```
AI Backend → UISpec JSON (with variant props) → DynamicRenderer → CVA → CSS Classes
```

### Backend Flow
```
ComponentTemplates → UIComponent (with variant props) → JSON Response
```

## Available Variants

### Button Variants

```typescript
import { buttonVariants, cn } from "../utils/componentVariants";

<button className={cn(buttonVariants({ variant: "primary", size: "large" }))}>
  Click Me
</button>
```

**Variants:**
- `variant`: `default` | `primary` | `secondary` | `danger` | `ghost` | `outline`
- `size`: `small` | `medium` | `large`
- `fullWidth`: `boolean`

**Example from Backend:**
```python
button = templates.button(
    id="submit-btn",
    text="Submit",
    on_click="form.submit",
    variant="primary",
    size="large"
)
```

### Input Variants

```typescript
<input className={cn(inputVariants({ variant: "filled", size: "large", error: true }))} />
```

**Variants:**
- `variant`: `default` | `filled` | `outline` | `underline`
- `size`: `small` | `medium` | `large`
- `error`: `boolean`
- `disabled`: `boolean`

### Text Variants

```typescript
<p className={cn(textVariants({ variant: "h1", weight: "bold", color: "accent" }))}>
  Heading
</p>
```

**Variants:**
- `variant`: `h1` | `h2` | `h3` | `body` | `caption` | `label`
- `weight`: `normal` | `medium` | `semibold` | `bold`
- `color`: `primary` | `secondary` | `accent` | `muted` | `error` | `success`
- `align`: `left` | `center` | `right`

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
- `layout`: `vertical` | `horizontal`
- `spacing`: `none` | `small` | `medium` | `large`
- `padding`: `none` | `small` | `medium` | `large`
- `align`: `start` | `center` | `end` | `stretch`
- `justify`: `start` | `center` | `end` | `between` | `around`

### Grid Variants

```typescript
<div className={cn(gridVariants({ columns: 4, spacing: "large", responsive: true }))}>
  {items}
</div>
```

**Variants:**
- `columns`: `1` | `2` | `3` | `4` | `5` | `6`
- `spacing`: `none` | `small` | `medium` | `large`
- `responsive`: `boolean`

### Card Variants (Launcher)

```typescript
<div className={cn(cardVariants({ variant: "elevated", hoverable: true, interactive: true }))}>
  Card content
</div>
```

**Variants:**
- `variant`: `default` | `elevated` | `outlined` | `ghost`
- `padding`: `none` | `small` | `medium` | `large`
- `hoverable`: `boolean`
- `interactive`: `boolean`

### Category Button Variants (Launcher)

```typescript
<button className={cn(categoryButtonVariants({ active: true, size: "large" }))}>
  Category
</button>
```

**Variants:**
- `active`: `boolean`
- `size`: `small` | `medium` | `large`

## Usage in DynamicRenderer

The `ComponentRenderer` automatically applies CVA variants based on props from the backend:

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

## Backend Template System

All component templates in `ai-service/src/agents/ui_generator.py` now support CVA variants:

```python
class ComponentTemplates:
    @staticmethod
    def button(
        id: str, 
        text: str, 
        on_click: Optional[str] = None,
        variant: str = "default",
        size: str = "medium"
    ) -> UIComponent:
        return UIComponent(
            type="button",
            id=id,
            props={"text": text, "variant": variant, "size": size},
            on_event={"click": on_click} if on_click else None
        )
```

## CSS Architecture

All variant classes are defined in component-specific CSS files:

- `DynamicRenderer.css` - Dynamic component variants
- `Launcher.css` - Launcher-specific variants
- `TitleBar.css` - Title bar control variants

Each variant class follows the pattern: `{component}-{variant}-{value}`

Example:
```css
.button-primary {
  background: linear-gradient(140deg, rgba(99, 102, 241, 0.25) 0%, rgba(139, 92, 246, 0.3) 100%);
  border-color: rgba(99, 102, 241, 0.6);
}
```

## Utility Functions

### `cn()` - Class Name Combiner

```typescript
import { cn } from "../utils/componentVariants";

<div className={cn(
  "base-class",
  conditionalClass && "conditional-class",
  variantFunction({ variant: "primary" })
)}>
```

### `extractVariantProps()` - Props Separator

```typescript
import { extractVariantProps } from "../utils/componentVariants";

const [variantProps, otherProps] = extractVariantProps(
  props,
  ["variant", "size", "fullWidth"]
);
```

## AI-Generated Components

When the AI generates UISpec JSON, it can now include variant props:

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

The `DynamicRenderer` will automatically apply the correct CVA variant classes.

## Adding New Variants

1. **Define in TypeScript:**
```typescript
// ui/src/utils/componentVariants.ts
export const newComponentVariants = cva("base-class", {
  variants: {
    variant: {
      default: "variant-default",
      special: "variant-special",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});
```

2. **Add CSS Classes:**
```css
/* Component.css */
.variant-default { /* styles */ }
.variant-special { /* styles */ }
```

3. **Update Backend Template:**
```python
# ui_generator.py
@staticmethod
def new_component(id: str, variant: str = "default") -> UIComponent:
    return UIComponent(
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
      {/* component */}
    </div>
  );
```

## Type Safety

CVA provides full TypeScript type inference:

```typescript
type ButtonVariants = VariantProps<typeof buttonVariants>;
// { variant?: "default" | "primary" | ..., size?: "small" | "medium" | "large", ... }
```

This ensures you can't pass invalid variant values at compile time.

## Benefits

✅ **Type Safety** - Invalid variants caught at compile time  
✅ **Consistency** - Same styling system across all components  
✅ **Maintainability** - Centralized variant definitions  
✅ **DX** - Auto-complete for all variant options  
✅ **Performance** - No runtime overhead, just class strings  
✅ **AI-Friendly** - Easy for LLM to generate correct variant props

## Examples

### Primary Call-to-Action Button
```typescript
buttonVariants({ variant: "primary", size: "large" })
```

### Error Input Field
```typescript
inputVariants({ variant: "outline", error: true })
```

### Centered Heading
```typescript
textVariants({ variant: "h1", weight: "bold", align: "center", color: "accent" })
```

### Responsive Grid
```typescript
gridVariants({ columns: 3, spacing: "large", responsive: true })
```

## References

- [CVA Documentation](https://cva.style/docs)
- [Tailwind CSS](https://tailwindcss.com)
- Component Variants: `ui/src/utils/componentVariants.ts`
- Backend Templates: `ai-service/src/agents/ui_generator.py`

