/**
 * AppGrid Component
 * Grid layout for app cards
 */

import React from 'react';
import type { AppMetadata } from '../types';
import { AppCard } from './AppCard';

interface AppGridProps {
  apps: AppMetadata[];
  favorites: Set<string>;
  onLaunch: (app: AppMetadata) => void;
  onToggleFavorite: (appId: string) => void;
  selectedIndex: number;
}

export const AppGrid: React.FC<AppGridProps> = ({
  apps,
  favorites,
  onLaunch,
  onToggleFavorite,
  selectedIndex,
}) => {
  return (
    <div className="app-grid">
      {apps.map((app, index) => (
        <AppCard
          key={app.id}
          app={app}
          isFavorite={favorites.has(app.id)}
          onLaunch={onLaunch}
          onToggleFavorite={onToggleFavorite}
          isSelected={index === selectedIndex}
        />
      ))}
    </div>
  );
};

