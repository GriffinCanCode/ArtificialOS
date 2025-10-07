/**
 * TypewriterText Component
 * Animated typewriter effect for text
 */

import React, { useEffect, useState } from "react";
import "./TypewriterText.css";

export interface TypewriterTextProps {
  text: string;
  speed?: number;
  delay?: number;
  cursor?: boolean;
  onComplete?: () => void;
  className?: string;
  style?: React.CSSProperties;
}

export const TypewriterText: React.FC<TypewriterTextProps> = ({
  text,
  speed = 50,
  delay = 0,
  cursor = true,
  onComplete,
  className = "",
  style = {},
}) => {
  const [displayText, setDisplayText] = useState("");
  const [isComplete, setIsComplete] = useState(false);

  useEffect(() => {
    let timeout: NodeJS.Timeout;
    let interval: NodeJS.Timeout;

    timeout = setTimeout(() => {
      let index = 0;
      interval = setInterval(() => {
        if (index < text.length) {
          setDisplayText(text.slice(0, index + 1));
          index++;
        } else {
          clearInterval(interval);
          setIsComplete(true);
          onComplete?.();
        }
      }, speed);
    }, delay);

    return () => {
      clearTimeout(timeout);
      clearInterval(interval);
    };
  }, [text, speed, delay, onComplete]);

  return (
    <span className={`typewriter-text ${className}`} style={style}>
      {displayText}
      {cursor && !isComplete && <span className="typewriter-cursor">|</span>}
    </span>
  );
};
