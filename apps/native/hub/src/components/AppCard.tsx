/**
 * AppCard Component
 * Individual app card in grid
 */

import React from 'react';
import type { AppMetadata } from '../types';

interface AppCardProps {
  app: AppMetadata;
  isFavorite: boolean;
  onLaunch: (app: AppMetadata) => void;
  onToggleFavorite: (appId: string) => void;
  isSelected?: boolean;
}

export const AppCard: React.FC<AppCardProps> = ({
  app,
  isFavorite,
  onLaunch,
  onToggleFavorite,
  isSelected = false,
}) => {
  const handleClick = (e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('.app-card-favorite')) {
      return; // Don't launch when clicking favorite button
    }
    onLaunch(app);
  };

  const handleFavoriteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onToggleFavorite(app.id);
  };

  return (
    <div
      className={`app-card ${isSelected ? 'selected' : ''}`}
      onClick={handleClick}
      title={app.description}
    >
      <button
        className={`app-card-favorite ${isFavorite ? 'active' : ''}`}
        onClick={handleFavoriteClick}
        title={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
      >
        {isFavorite ? '⭐' : '☆'}
      </button>

      <div className="app-card-icon">{app.icon}</div>
      <div className="app-card-name">{app.name}</div>
      <div className="app-card-description">{app.description}</div>

      <div className="app-card-footer">
        <span className="app-card-type">{getTypeLabel(app.type)}</span>
        <span className="app-card-category">{app.category}</span>
      </div>
    </div>
  );
};

function getTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    blueprint: 'Blueprint',
    native_web: 'Native',
    native_proc: 'Process',
  };
  return labels[type] || type;
}

