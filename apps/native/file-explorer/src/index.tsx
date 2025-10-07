/**
 * File Explorer Entry Point
 * Exports the main app component
 */

import App from './App';
import type { NativeAppProps } from './sdk';

export default function FileExplorer(props: NativeAppProps) {
  return <App {...props} />;
}
