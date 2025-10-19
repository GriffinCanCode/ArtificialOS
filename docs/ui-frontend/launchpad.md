# Launchpad Feature

A macOS-style launchpad that seamlessly replaces the desktop icons with a beautiful app grid when activated.

## Features

- **Desktop Integration**: Replaces desktop icons area while keeping menubar and dock visible
- **Smooth Transitions**: Desktop icons fade out as launchpad apps fade in with scale animations
- **Search Functionality**: Real-time search to filter apps by name, description, or category
- **Keyboard Shortcuts**: 
  - `L` - Toggle launchpad
  - `ESC` - Close launchpad
  - Type to search apps
- **Click to Launch**: Click any app to launch it immediately
- **Persistent Background**: Desktop background, gradient, menubar, and dock stay visible
- **Responsive Design**: Adapts to different screen sizes

## Usage

### Opening the Launchpad

There are three ways to open the launchpad:

1. **Keyboard Shortcut**: Press `L` (Cmd+L on Mac, Ctrl+L on Windows/Linux)
2. **Menu Bar Button**: Click the ⚡ lightning bolt icon in the menu bar
3. **Programmatically**: Call `setShowLaunchpad(true)` in the app

### Closing the Launchpad

1. **Keyboard**: Press `ESC`
2. **Button**: Click the ⚡ lightning bolt button again
3. **Launch App**: Optionally auto-closes after launching an app (currently disabled)

## Implementation Details

### Components

- **`Launchpad.tsx`**: Main component that fetches and displays apps
- **`Launchpad.css`**: Styling with smooth animations and responsive design
- **`Desktop.tsx`**: Integrates launchpad and handles icon visibility transitions

### Key Features

1. **Dynamic App Loading**: Fetches apps from the registry API when opened
2. **Fuzzy Search**: Real-time filtering of apps based on search query
3. **Animation Stagger**: Each app card animates in with a slight delay for a cascading effect
4. **Seamless Transition**: Desktop icons and launchpad cross-fade with scale transforms
5. **Keyboard Navigation**: Full keyboard support for accessibility
6. **Active State**: Lightning bolt button glows when launchpad is active

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
      "icon": "",
      "description": "App description",
      "category": "productivity",
      "type": "blueprint"
    }
  ]
}
```

## Styling

The launchpad uses the following design tokens:

- **Integration**: Absolutely positioned within desktop, below menubar
- **App Cards**: Glass-morphism style with backdrop blur and hover effects
- **Animations**: Cubic bezier easing for smooth cross-fade transitions
- **Grid**: Responsive with `auto-fill` and `minmax(140px, 1fr)`
- **Desktop Icons**: Fade out with opacity and scale when launchpad activates

### Customization

You can customize the appearance by modifying `Launchpad.css`:

- **Grid columns**: Change `grid-template-columns` in `.launchpad-grid`
- **Animation speed**: Modify `transition` duration in `.launchpad`
- **Card size**: Adjust `minmax` values in the grid
- **Colors**: Update RGBA values for different backgrounds

## Accessibility

- **Keyboard shortcuts**: Full keyboard navigation support
- **Focus management**: Auto-focus on search input when opened
- **ESC key**: Standard behavior to close overlay
- **Click outside**: Click anywhere to close
- **Screen readers**: Semantic HTML with proper ARIA labels

## Performance

- **Lazy loading**: Only fetches apps when launchpad is opened
- **Optimized animations**: Uses transform and opacity for 60fps animations
- **Backdrop filter**: Hardware-accelerated for smooth performance
- **Efficient re-renders**: Only updates when search query or apps change

## Future Enhancements

Potential improvements for the launchpad:

1. **Categories**: Filter apps by category (productivity, utilities, etc.)
2. **Favorites**: Pin favorite apps to the top
3. **Drag to Reorder**: Customize app order
4. **App Info**: Show more details on hover or long-press
5. **Recent Apps**: Show recently used apps first
6. **Folders**: Group apps into folders
7. **Multi-page**: Support pagination for many apps
8. **Gestures**: Pinch to close, swipe between pages

## Browser Compatibility

- ✅ Chrome/Edge 88+ (full support)
- ✅ Firefox 87+ (full support)
- ✅ Safari 14+ (full support with `-webkit-` prefixes)
- ⚠️ Older browsers: Fallback to solid background (no blur)

## Related Files

- [`ui/src/ui/components/layout/Launchpad.tsx`](../ui/src/ui/components/layout/Launchpad.tsx)
- [`ui/src/ui/components/layout/Launchpad.css`](../ui/src/ui/components/layout/Launchpad.css)
- [`ui/src/ui/components/layout/Desktop.tsx`](../ui/src/ui/components/layout/Desktop.tsx)
- [`ui/src/app/App.tsx`](../ui/src/app/App.tsx)

