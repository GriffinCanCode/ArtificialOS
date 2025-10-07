/**
 * CustomText Component
 * Renders text using OpenType.js with advanced typography features
 */

import React, { useEffect, useRef, useState } from 'react';
import { typography, TypographyOptions, typographyPresets } from '../../../core/utils/typography';
import './CustomText.css';

export interface CustomTextProps extends TypographyOptions {
  text: string;
  fontName?: string;
  preset?: keyof typeof typographyPresets;
  className?: string;
  style?: React.CSSProperties;
  fill?: string;
  stroke?: string;
  strokeWidth?: number;
  animate?: 'fadeIn' | 'slideUp' | 'draw' | 'morph' | 'glitch' | 'none';
  animationDuration?: number;
  gradient?: {
    colors: string[];
    angle?: number;
  };
  shadow?: {
    color: string;
    blur: number;
    offsetX?: number;
    offsetY?: number;
  };
  glow?: {
    color: string;
    intensity?: number;
  };
  onLoad?: () => void;
}

export const CustomText: React.FC<CustomTextProps> = ({
  text,
  fontName = 'system',
  preset,
  className = '',
  style = {},
  fill = 'currentColor',
  stroke,
  strokeWidth = 0,
  animate = 'none',
  animationDuration = 1,
  gradient,
  shadow,
  glow,
  onLoad,
  ...typographyOptions
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const [pathData, setPathData] = useState<string | null>(null);
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
  const [isLoaded, setIsLoaded] = useState(false);

  // Merge preset with custom options
  const options = preset
    ? { ...typographyPresets[preset], ...typographyOptions }
    : typographyOptions;

  useEffect(() => {
    const loadAndRender = async () => {
      try {
        // Ensure font is loaded
        const font = typography.getFont(fontName);
        if (!font) {
          console.warn(`Font ${fontName} not loaded, using fallback`);
          setIsLoaded(true);
          return;
        }

        // Generate path data
        const path = typography.textToPath(text, fontName, {
          ...options,
          x: 0,
          y: options.fontSize || 72,
        });

        if (path) {
          setPathData(path);

          // Measure text for SVG viewBox
          const measurements = typography.measureText(
            text,
            fontName,
            options.fontSize || 72
          );
          setDimensions({
            width: measurements.width,
            height: measurements.height,
          });

          setIsLoaded(true);
          onLoad?.();
        }
      } catch (error) {
        console.error('Error rendering custom text:', error);
        setIsLoaded(true);
      }
    };

    loadAndRender();
  }, [text, fontName, JSON.stringify(options), onLoad]);

  // Generate gradient ID for uniqueness
  const gradientId = `text-gradient-${Math.random().toString(36).substr(2, 9)}`;
  const shadowId = `text-shadow-${Math.random().toString(36).substr(2, 9)}`;
  const glowId = `text-glow-${Math.random().toString(36).substr(2, 9)}`;

  if (!isLoaded || !pathData) {
    // Fallback to regular text while loading
    return (
      <div
        className={`custom-text-fallback ${className}`}
        style={{
          fontSize: options.fontSize || 72,
          ...style,
        }}
      >
        {text}
      </div>
    );
  }

  const animationClass = animate !== 'none' ? `animate-${animate}` : '';

  return (
    <svg
      ref={svgRef}
      className={`custom-text ${animationClass} ${className}`}
      viewBox={`0 0 ${dimensions.width} ${dimensions.height * 1.2}`}
      style={{
        width: '100%',
        height: 'auto',
        overflow: 'visible',
        ...style,
      }}
    >
      <defs>
        {/* Gradient definition */}
        {gradient && (
          <linearGradient
            id={gradientId}
            gradientTransform={`rotate(${gradient.angle || 0})`}
          >
            {gradient.colors.map((color, index) => (
              <stop
                key={index}
                offset={`${(index / (gradient.colors.length - 1)) * 100}%`}
                stopColor={color}
              />
            ))}
          </linearGradient>
        )}

        {/* Shadow filter */}
        {shadow && (
          <filter id={shadowId} x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur in="SourceAlpha" stdDeviation={shadow.blur} />
            <feOffset dx={shadow.offsetX || 0} dy={shadow.offsetY || 2} result="offsetblur" />
            <feFlood floodColor={shadow.color} />
            <feComposite in2="offsetblur" operator="in" />
            <feMerge>
              <feMergeNode />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        )}

        {/* Glow filter */}
        {glow && (
          <filter id={glowId} x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation={glow.intensity || 4} result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        )}
      </defs>

      {/* Render the text path */}
      <g
        dangerouslySetInnerHTML={{ __html: pathData }}
        fill={gradient ? `url(#${gradientId})` : fill}
        stroke={stroke}
        strokeWidth={strokeWidth}
        filter={
          shadow ? `url(#${shadowId})` : glow ? `url(#${glowId})` : undefined
        }
        style={{
          animationDuration: `${animationDuration}s`,
        }}
      />

      {/* Animation styles */}
      <style>{`
        .animate-fadeIn g {
          animation: customTextFadeIn ${animationDuration}s ease-out forwards;
        }

        .animate-slideUp g {
          animation: customTextSlideUp ${animationDuration}s ease-out forwards;
        }

        .animate-draw path {
          stroke-dasharray: 1000;
          stroke-dashoffset: 1000;
          animation: customTextDraw ${animationDuration}s ease-out forwards;
        }

        .animate-glitch g {
          animation: customTextGlitch ${animationDuration}s ease-in-out infinite;
        }

        @keyframes customTextFadeIn {
          from {
            opacity: 0;
            transform: scale(0.95);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }

        @keyframes customTextSlideUp {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        @keyframes customTextDraw {
          to {
            stroke-dashoffset: 0;
          }
        }

        @keyframes customTextGlitch {
          0%, 100% {
            transform: translate(0, 0);
            filter: hue-rotate(0deg);
          }
          20% {
            transform: translate(-2px, 2px);
            filter: hue-rotate(20deg);
          }
          40% {
            transform: translate(2px, -2px);
            filter: hue-rotate(-20deg);
          }
          60% {
            transform: translate(-2px, -2px);
            filter: hue-rotate(20deg);
          }
          80% {
            transform: translate(2px, 2px);
            filter: hue-rotate(-20deg);
          }
        }
      `}</style>
    </svg>
  );
};
