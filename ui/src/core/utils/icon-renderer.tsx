/**
 * Icon Renderer Utility
 * Handles rendering of both emoji and SVG/image icons
 */

import React from 'react';

export interface IconRendererProps {
  icon: string;
  alt?: string;
  size?: number;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * Renders an icon - either as an emoji (text) or as an image (SVG/PNG/etc)
 */
export const IconRenderer: React.FC<IconRendererProps> = ({
  icon,
  alt = 'Icon',
  size = 48,
  className,
  style
}) => {
  // Determine if icon is a file path (SVG/image) or emoji
  const isImagePath = icon.startsWith('/') || icon.startsWith('http');

  if (isImagePath) {
    return (
      <img
        src={icon}
        alt={alt}
        className={className}
        style={{
          width: `${size}px`,
          height: `${size}px`,
          objectFit: 'contain',
          ...style
        }}
      />
    );
  }

  // Render as emoji/text
  return (
    <span
      className={className}
      style={{
        fontSize: `${size}px`,
        lineHeight: `${size}px`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        ...style
      }}
    >
      {icon}
    </span>
  );
};

/**
 * Hook to determine if an icon is an image path
 */
export const useIsImageIcon = (icon: string): boolean => {
  return icon.startsWith('/') || icon.startsWith('http');
};
