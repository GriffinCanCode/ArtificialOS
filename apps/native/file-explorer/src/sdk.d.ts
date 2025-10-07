/**
 * SDK Type Declarations
 * Types for native app SDK integration
 */

import type { ComponentState } from '../../../../ui/src/features/dynamics/state/state';
import type { ToolExecutor } from '../../../../ui/src/features/dynamics/execution/executor';

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

export type { ComponentState, ToolExecutor };
