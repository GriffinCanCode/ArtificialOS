# Launchpad Feature

A desktop app grid that integrates with the desktop environment and replaces desktop icons when activated.

## Overview

The launchpad provides quick access to installed applications through a searchable grid interface. It integrates with the desktop component and can be toggled via keyboard shortcut or menu bar button.

## Features

- **Desktop Integration** - Replaces desktop icons area while preserving background, menu bar, and dock
- **Smooth Animations** - Scale and opacity transitions for opening/closing
- **Search** - Real-time filtering by app name, description, or category
- **Keyboard Support** - Cmd+L (Mac) / Ctrl+L (Windows/Linux) to toggle
- **Click to Launch** - Select and launch apps directly
- **Loading States** - Visual feedback while fetching apps
- **Empty States** - Graceful handling when no apps are available

## Usage

### Opening the Launchpad

Three methods are available:

1. **Keyboard Shortcut** - Cmd+L (Mac) or Ctrl+L (Windows/Linux)
2. **Menu Bar** - Click the lightning bolt icon in the menu bar
3. **Programmatically** - Call `onToggleLaunchpad()` from parent component

### Closing the Launchpad

1. **Keyboard** - Press ESC or Cmd+L again
2. **Menu Bar Button** - Click the lightning bolt button
3. **Click Outside** - Click on the background to dismiss

### Searching Apps

Type in the search bar to filter by:
- App name
- App description
- App category

## Implementation

### Components

- **`Launchpad.tsx`** - Main component that fetches and displays apps
- **`Launchpad.css`** - Styling with animations
- **`Desktop.tsx`** - Integrates launchpad with desktop environment

### Key Implementation Details

1. **Dynamic App Loading** - Apps are fetched from the backend when the launchpad is opened
2. **Search Filtering** - Performed locally for instant response
3. **Animation Stagger** - Each app card receives an animation delay based on its index
4. **State Management** - Uses React hooks for visibility, search query, and app list
5. **Error Handling** - Gracefully handles fetch failures and displays empty state

### API Integration

The launchpad fetches apps from:

```
GET http://localhost:8000/registry/apps
```

Expected response format:

```json
{
  "apps": [
    {
      "id": "app-id",
      "name": "App Name",
      "icon": "icon-string",
      "description": "App description",
      "category": "productivity",
      "type": "blueprint"
    }
  ]
}
```

## Styling

The launchpad uses absolute positioning within the desktop viewport. Key CSS properties:

- **Position** - Absolutely positioned below the menu bar (36px offset)
- **Visibility** - Toggled via opacity and pointer-events
- **Animation** - 0.5s cubic bezier easing with scale transform
- **Grid** - Responsive with `auto-fill` and `minmax(140px, 1fr)` for flexible columns
- **Search Bar** - Glass morphism effect with blur and backdrop filter
- **App Cards** - Hover effects with scale, shadow, and icon transformation

### Customization

Styling can be adjusted in `Launchpad.css`:

- **Grid columns** - Modify `minmax()` values in `.launchpad-grid`
- **Animation speed** - Adjust `transition` durations
- **Card size** - Change grid sizing and padding values
- **Colors** - Update RGBA values for backgrounds and borders

## Performance Considerations

- Apps are fetched only when the launchpad is opened
- Search filtering runs locally for immediate results
- CSS animations use `transform` and `opacity` for GPU acceleration
- Custom scrollbar styling for visual consistency
- Staggered animation delays prevent visual complexity

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+L / Ctrl+L | Toggle launchpad visibility |
| ESC | Close launchpad |
| Type | Search for apps |

## Component Props

```typescript
interface LaunchpadProps {
  isVisible: boolean;           // Current visibility state
  onLaunchApp: (appId: string) => void;  // Callback when app is selected
}
```

## Related Components

- `Desktop.tsx` - Parent component integrating launchpad
- `DockItem.tsx` - Related dock interface for quick app access
- `IconGrid.tsx` - Alternative icon-based app access

## Files

- Implementation: `ui/src/ui/components/layout/Launchpad.tsx`
- Styling: `ui/src/ui/components/layout/Launchpad.css`
- Integration: `ui/src/ui/components/layout/Desktop.tsx`
- API: `backend/internal/api/registry.go`

