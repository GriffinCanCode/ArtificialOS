# TanStack Query Integration - Complete Setup

## Overview

Successfully integrated **TanStack Query** (React Query) into the frontend for all Registry and Session API calls. This provides automatic caching, background refetching, optimistic updates, and better state management.

---

## 🎯 What Was Implemented

### 1. **Core Setup**

#### Installed Package
```bash
npm install @tanstack/react-query
```

#### Query Client Configuration
**File:** `ui/src/lib/queryClient.ts`

- Centralized QueryClient with sensible defaults
- 5-minute cache time for unused data
- Automatic refetch on reconnect
- Disabled refetch on window focus (can be overridden per query)

#### Provider Setup
**File:** `ui/src/renderer/App.tsx`

```tsx
<QueryClientProvider client={queryClient}>
  <WebSocketProvider>
    <AppContent />
  </WebSocketProvider>
</QueryClientProvider>
```

---

### 2. **Registry API Integration**

#### Custom Hooks
**File:** `ui/src/hooks/useRegistryQueries.ts`

##### Query Hooks (Read Operations)
- **`useRegistryApps(category?)`** - List all registry apps
  - ✅ Auto-cached for 30 seconds
  - ✅ Background refetching
  - ✅ Supports category filtering
  - ✅ Automatic deduplication
  
- **`useRegistryApp(packageId)`** - Get specific app details
  - ✅ Cached for 60 seconds
  - ✅ Can be enabled/disabled conditionally

##### Mutation Hooks (Write Operations)
- **`useLaunchApp()`** - Launch app from registry
  - ✅ Logs success/errors automatically
  
- **`useSaveApp()`** - Save running app to registry
  - ✅ Invalidates apps list after save
  - ✅ Cache automatically refreshes
  
- **`useDeleteApp()`** - Delete app from registry
  - ✅ **Optimistic updates** - UI updates immediately
  - ✅ Automatic rollback on error
  - ✅ Refetches to ensure consistency

##### Convenience Hook
- **`useRegistryMutations()`** - All mutations in one hook

#### Key Features
```typescript
// Centralized query keys for consistency
export const registryKeys = {
  all: ["registry"],
  apps: () => [...registryKeys.all, "apps"],
  appsList: (category?: string) => [...registryKeys.apps(), "list", { category }],
  app: (id: string) => [...registryKeys.apps(), "detail", id],
};
```

---

### 3. **Session API Integration**

#### Custom Hooks
**File:** `ui/src/hooks/useSessionQueries.ts`

##### Query Hooks (Read Operations)
- **`useSessions()`** - List all sessions
  - ✅ Auto-sorted by most recent first
  - ✅ Cached for 10 seconds
  - ✅ Background refetching
  
- **`useSession(sessionId)`** - Get specific session details
  - ✅ Cached for 30 seconds
  - ✅ Conditional fetching support

##### Mutation Hooks (Write Operations)
- **`useSaveSession()`** - Save session with custom name
  - ✅ Optimistically updates cache
  - ✅ Invalidates list for consistency
  
- **`useSaveDefaultSession()`** - Auto-save with default name
  - ✅ Lightweight background operation
  - ✅ Silently marks cache as stale
  
- **`useRestoreSession()`** - Restore saved session
  - ✅ Updates application state
  - ✅ Caches restored session data
  
- **`useDeleteSession()`** - Delete saved session
  - ✅ **Optimistic updates** - instant UI feedback
  - ✅ Automatic rollback on error
  - ✅ Refetches for consistency

##### Convenience Hook
- **`useSessionMutations()`** - All mutations in one hook

---

### 4. **Component Updates**

#### Launcher Component
**File:** `ui/src/components/Launcher.tsx`

**Before:**
```tsx
const [apps, setApps] = useState([]);
const [loading, setLoading] = useState(true);

const loadApps = async () => {
  const response = await RegistryClient.listApps();
  setApps(response.apps);
};
```

**After:**
```tsx
const { data, isLoading, error, refetch } = useRegistryApps(category);
const { launchApp, deleteApp } = useRegistryMutations();

const apps = data?.apps ?? [];
```

**Benefits:**
- ✅ Automatic caching - no redundant API calls
- ✅ Category changes use cached data when available
- ✅ Loading/error states managed automatically
- ✅ Optimistic deletes with instant UI feedback

---

#### DynamicRenderer Component
**File:** `ui/src/components/DynamicRenderer.tsx`

**Before:**
```tsx
const [isSavingApp, setIsSavingApp] = useState(false);

const handleSaveApp = async (data) => {
  setIsSavingApp(true);
  try {
    await RegistryClient.saveApp(request);
  } finally {
    setIsSavingApp(false);
  }
};
```

**After:**
```tsx
const saveAppMutation = useSaveApp();

const handleSaveApp = async (data) => {
  await saveAppMutation.mutateAsync(request);
};
```

**Benefits:**
- ✅ Loading state from `saveAppMutation.isPending`
- ✅ Error handling built-in
- ✅ Cache invalidation automatic
- ✅ Cleaner, less boilerplate code

---

#### useSessionManager Hook
**File:** `ui/src/hooks/useSessionManager.ts`

**Before:**
```tsx
const [isSaving, setIsSaving] = useState(false);
const [error, setError] = useState(null);

const save = async (name, description) => {
  setIsSaving(true);
  setError(null);
  try {
    await SessionClient.saveSession(request);
  } catch (err) {
    setError(err.message);
  } finally {
    setIsSaving(false);
  }
};
```

**After:**
```tsx
const saveSessionMutation = useSaveSession();
const { data: sessionsData } = useSessions();

const save = async (name, description) => {
  return await saveSessionMutation.mutateAsync(request);
};

const isSaving = saveSessionMutation.isPending;
const error = saveSessionMutation.error?.message;
```

**Benefits:**
- ✅ State derived from mutations
- ✅ Auto-restore uses cached sessions list
- ✅ No manual state management
- ✅ Automatic cache updates

---

#### TitleBar Component
**File:** `ui/src/components/TitleBar.tsx`

**Before:**
```tsx
const [sessions, setSessions] = useState([]);

const handleShowSessions = async () => {
  const result = await SessionClient.listSessions();
  setSessions(result.sessions);
};
```

**After:**
```tsx
const { data, refetch, isLoading } = useSessions();
const deleteSessionMutation = useDeleteSession();

const sessions = data?.sessions ?? [];

const handleShowSessions = () => {
  refetch(); // Instant if cached, background refetch
};
```

**Benefits:**
- ✅ Cached sessions load instantly
- ✅ Background refetch ensures freshness
- ✅ Delete with optimistic UI updates
- ✅ Automatic loading states
- ✅ Added delete session functionality

---

## 🎨 UI Enhancements

### Session Delete Button
Added delete button to session items with smooth animations:

**CSS:** `ui/src/components/TitleBar.css`

```css
.session-delete {
  opacity: 0; /* Hidden by default */
  transition: all 0.2s ease;
}

.session-item:hover .session-delete {
  opacity: 1; /* Fade in on hover */
}
```

- ✅ Only visible on hover
- ✅ Red color scheme for delete action
- ✅ Disabled state during deletion
- ✅ Smooth fade-in animation

---

## 📊 Benefits Summary

### Performance
- **Reduced API calls** - Data cached and reused
- **Instant UI feedback** - Optimistic updates
- **Background refetching** - Always fresh without blocking UI
- **Request deduplication** - Multiple components share cache

### Developer Experience
- **Less boilerplate** - No manual loading/error state management
- **Type-safe** - Full TypeScript support
- **Centralized query keys** - Easy cache invalidation
- **Built-in retry logic** - Automatic exponential backoff

### User Experience
- **Faster perceived performance** - Cached data loads instantly
- **Optimistic updates** - Delete/save feels immediate
- **Loading states** - Better feedback during operations
- **Error recovery** - Automatic rollback on failures

---

## 🔧 Configuration

### Stale Times (Data Freshness)

```typescript
// Registry
useRegistryApps: 30 seconds   // Frequently accessed
useRegistryApp: 60 seconds    // Less frequently accessed

// Sessions
useSessions: 10 seconds       // Moderately fresh
useSession: 30 seconds        // Individual session details
```

### Cache Times (Garbage Collection)

```typescript
// Default: 5 minutes for all queries
gcTime: 5 * 60 * 1000
```

### Retry Logic

```typescript
// Registry: 2 retries with exponential backoff
retry: 2
retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000)

// Sessions: 2 retries
retry: 2
```

---

## 🚀 Usage Examples

### Example 1: Fetching Registry Apps
```tsx
function MyComponent() {
  const { data, isLoading, error, refetch } = useRegistryApps("productivity");
  
  if (isLoading) return <Spinner />;
  if (error) return <Error message={error.message} />;
  
  return (
    <div>
      {data.apps.map(app => <AppCard key={app.id} app={app} />)}
      <button onClick={() => refetch()}>Refresh</button>
    </div>
  );
}
```

### Example 2: Deleting with Optimistic Update
```tsx
function AppCard({ app }) {
  const { deleteApp } = useRegistryMutations();
  
  const handleDelete = () => {
    // UI updates immediately, rolls back on error
    deleteApp.mutate(app.id, {
      onSuccess: () => console.log("Deleted!"),
      onError: () => alert("Failed to delete"),
    });
  };
  
  return (
    <div>
      {app.name}
      <button onClick={handleDelete} disabled={deleteApp.isPending}>
        {deleteApp.isPending ? "Deleting..." : "Delete"}
      </button>
    </div>
  );
}
```

### Example 3: Saving Session
```tsx
function SaveButton() {
  const saveSessionMutation = useSaveSession();
  
  const handleSave = async () => {
    await saveSessionMutation.mutateAsync({
      name: "My Session",
      description: "Work in progress",
      chat_state: {...},
      ui_state: {...},
    });
  };
  
  return (
    <button 
      onClick={handleSave}
      disabled={saveSessionMutation.isPending}
    >
      {saveSessionMutation.isPending ? "Saving..." : "Save Session"}
    </button>
  );
}
```

---

## 🔍 Debugging

### DevTools (Optional)
To add React Query DevTools for debugging:

```bash
npm install @tanstack/react-query-devtools
```

```tsx
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <YourApp />
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}
```

### Console Logging
All queries and mutations log to console via the logger system:

```typescript
logger.info("Fetching registry apps", { category });
logger.error("Failed to save app", error);
```

---

## ✅ Testing

### Build Status
```bash
npm run build
# ✓ built in 1.20s - NO ERRORS
```

### Type Safety
- ✅ All hooks fully typed
- ✅ No TypeScript errors
- ✅ Inference works correctly

---

## 📝 Next Steps (Optional Enhancements)

1. **Add React Query DevTools** - For visual debugging
2. **Implement prefetching** - Load data before user requests it
3. **Add pagination** - For large registry/session lists
4. **Infinite queries** - For infinite scroll
5. **Persist cache** - Save to localStorage between sessions

---

## 📚 Resources

- [TanStack Query Docs](https://tanstack.com/query/latest)
- [Query Keys Best Practices](https://tkdodo.eu/blog/effective-react-query-keys)
- [Optimistic Updates Guide](https://tanstack.com/query/latest/docs/react/guides/optimistic-updates)

---

## 🎉 Complete Integration

**Registry API:** ✅ Fully migrated  
**Session API:** ✅ Fully migrated  
**Components:** ✅ All updated  
**Type Safety:** ✅ 100% coverage  
**Build:** ✅ No errors  
**UI/UX:** ✅ Enhanced with optimistic updates  

Your frontend now has production-ready data fetching with TanStack Query! 🚀

