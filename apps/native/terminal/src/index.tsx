/**
 * Terminal Entry Point
 * Exports the main app component
 */

import App from './App';
import type { NativeAppProps } from './sdk';

export default function Terminal(props: NativeAppProps) {
  return <App {...props} />;
}
