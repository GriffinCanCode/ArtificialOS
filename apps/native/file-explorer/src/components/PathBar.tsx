/**
 * Path Bar Component
 * Breadcrumb navigation
 */

import { useMemo } from 'react';
import { parsePath } from '../utils';

interface PathBarProps {
  currentPath: string;
  onNavigate: (path: string) => void;
}

export function PathBar({ currentPath, onNavigate }: PathBarProps) {
  const segments = useMemo(() => parsePath(currentPath), [currentPath]);

  return (
    <div className="path-bar">
      <div className="path-segments">
        {segments.map((segment, index) => (
          <span key={segment.path} className="path-segment">
            {index > 0 && <span className="path-separator">/</span>}
            <button
              className="path-button"
              onClick={() => onNavigate(segment.path)}
              title={segment.path}
            >
              {segment.name}
            </button>
          </span>
        ))}
      </div>
      <div className="path-full" title={currentPath}>
        {currentPath}
      </div>
    </div>
  );
}
