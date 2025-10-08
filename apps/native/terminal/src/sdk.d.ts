/**
 * SDK Type Declarations
 * Types for native app SDK integration
 */

export interface ComponentState {
  get(key: string, defaultValue?: any): any;
  set(key: string, value: any): void;
  subscribe(key: string, callback: () => void): () => void;
}

export interface ToolExecutor {
  execute(toolId: string, params?: any): Promise<any>;
}

export interface NativeAppContext {
  appId: string;
  state: ComponentState;
  executor: ToolExecutor;
  window: AppWindow;
}

export interface AppWindow {
  id: string;
  setTitle: (title: string) => void;
  setIcon: (icon: string) => void;
  close: () => void;
  minimize: () => void;
  maximize: () => void;
  focus: () => void;
}

export interface NativeAppProps {
  context: NativeAppContext;
}

