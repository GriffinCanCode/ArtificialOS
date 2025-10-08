/**
 * EmptyState Component
 * Displayed when no apps are found
 */

import React from 'react';

interface EmptyStateProps {
  message?: string;
  icon?: string;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  message = 'No apps found',
  icon = '📦',
}) => {
  return (
    <div className="empty-state">
      <div className="empty-icon">{icon}</div>
      <div className="empty-message">{message}</div>
    </div>
  );
};

