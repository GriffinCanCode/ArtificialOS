/**
 * Custom type declarations for tinykeys
 * Fixes module resolution issues with package.json exports field
 */

declare module "tinykeys" {
  export type KeyBindingPress = [mods: string[], key: string | RegExp];

  export interface KeyBindingMap {
    [keybinding: string]: (event: KeyboardEvent) => void;
  }

  export interface KeyBindingHandlerOptions {
    timeout?: number;
  }

  export interface KeyBindingOptions extends KeyBindingHandlerOptions {
    event?: "keydown" | "keyup";
    capture?: boolean;
  }

  export function parseKeybinding(str: string): KeyBindingPress[];

  export function matchKeyBindingPress(
    event: KeyboardEvent,
    press: KeyBindingPress
  ): boolean;

  export function createKeybindingsHandler(
    keyBindingMap: KeyBindingMap,
    options?: KeyBindingHandlerOptions
  ): (event: Event) => void;

  export function tinykeys(
    target: Window | HTMLElement,
    keyBindingMap: KeyBindingMap,
    options?: KeyBindingOptions
  ): () => void;
}
