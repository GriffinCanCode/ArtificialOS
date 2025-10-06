/**
 * Shared types for tool executors
 */

import { ComponentState } from "../../../state/state";

export interface ExecutorContext {
  componentState: ComponentState;
  appId: string | null;
}

export interface BaseExecutor {
  execute(action: string, params: Record<string, any>): any;
}

export interface AsyncExecutor {
  execute(action: string, params: Record<string, any>): Promise<any>;
}
