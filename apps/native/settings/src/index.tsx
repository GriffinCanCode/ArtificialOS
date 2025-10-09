import type { NativeAppProps } from './sdk';
import App from './App';

/**
 * Entry point for Settings
 * This is the component that will be loaded by the OS
 */
export default function SettingsApp(props: NativeAppProps) {
  return <App {...props} />;
}
