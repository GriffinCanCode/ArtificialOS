/**
 * TypographyShowcase Component
 * Demonstrates all typography features
 * Use this in development to test typography effects
 */

import React, { useState } from "react";
import { CustomText } from "./CustomText";
import { AnimatedTitle } from "./AnimatedTitle";
import { BrandLogo } from "./BrandLogo";
import { WindowTitleBar } from "./WindowTitleBar";
import { TypewriterText } from "./TypewriterText";
import "./TypographyShowcase.css";

export const TypographyShowcase: React.FC = () => {
  const [activeEffect, setActiveEffect] = useState<string>("gradient");

  return (
    <div className="typography-showcase">
      <div className="showcase-section">
        <h2>Brand Logo</h2>
        <div className="showcase-row">
          <BrandLogo size="small" />
          <BrandLogo size="medium" />
          <BrandLogo size="large" />
        </div>
      </div>

      <div className="showcase-section">
        <h2>Animated Titles</h2>
        <div className="showcase-grid">
          <div className="showcase-item">
            <label>Gradient Effect</label>
            <AnimatedTitle text="Beautiful Typography" preset="title" effect="gradient" />
          </div>
          <div className="showcase-item">
            <label>Glow Effect</label>
            <AnimatedTitle text="Glowing Text" preset="title" effect="glow" />
          </div>
          <div className="showcase-item">
            <label>Shimmer Effect</label>
            <AnimatedTitle text="Shimmering" preset="title" effect="shimmer" />
          </div>
          <div className="showcase-item">
            <label>Wave Effect</label>
            <AnimatedTitle text="Wave Motion" preset="title" effect="wave" />
          </div>
          <div className="showcase-item">
            <label>Split Effect</label>
            <AnimatedTitle text="Character Split" preset="title" effect="split" />
          </div>
          <div className="showcase-item">
            <label>Typewriter</label>
            <AnimatedTitle text="Typing Animation" preset="title" effect="typewriter" />
          </div>
        </div>
      </div>

      <div className="showcase-section">
        <h2>Typography Presets</h2>
        <div className="showcase-presets">
          <div className="preset-demo">
            <AnimatedTitle text="Display" preset="display" effect="gradient" />
            <span className="preset-label">Display (128px)</span>
          </div>
          <div className="preset-demo">
            <AnimatedTitle text="Hero" preset="hero" effect="gradient" />
            <span className="preset-label">Hero (96px)</span>
          </div>
          <div className="preset-demo">
            <AnimatedTitle text="Title" preset="title" effect="gradient" />
            <span className="preset-label">Title (48px)</span>
          </div>
          <div className="preset-demo">
            <AnimatedTitle text="Heading" preset="heading" effect="gradient" />
            <span className="preset-label">Heading (32px)</span>
          </div>
        </div>
      </div>

      <div className="showcase-section">
        <h2>Typewriter Effect</h2>
        <div className="typewriter-demo">
          <TypewriterText
            text="This text appears character by character, just like a classic typewriter!"
            speed={50}
            cursor={true}
          />
        </div>
      </div>

      <div className="showcase-section">
        <h2>Window Title Bars</h2>
        <div className="showcase-col">
          <WindowTitleBar title="Active Window" icon="📦" active={true} />
          <WindowTitleBar title="Inactive Window" icon="📄" active={false} />
        </div>
      </div>

      <div className="showcase-section">
        <h2>Interactive Demo</h2>
        <div className="interactive-controls">
          <button
            className={activeEffect === "gradient" ? "active" : ""}
            onClick={() => setActiveEffect("gradient")}
          >
            Gradient
          </button>
          <button
            className={activeEffect === "glow" ? "active" : ""}
            onClick={() => setActiveEffect("glow")}
          >
            Glow
          </button>
          <button
            className={activeEffect === "shimmer" ? "active" : ""}
            onClick={() => setActiveEffect("shimmer")}
          >
            Shimmer
          </button>
        </div>
        <div className="interactive-preview">
          <AnimatedTitle
            text="Interactive Typography"
            preset="hero"
            effect={activeEffect as any}
            key={activeEffect}
          />
        </div>
      </div>

      <div className="showcase-section">
        <h2>Color Combinations</h2>
        <div className="showcase-grid">
          <div className="color-demo">
            <CustomText
              text="Purple Pink"
              preset="title"
              gradient={{
                colors: ["#8B5CF6", "#EC4899"],
                angle: 45,
              }}
            />
          </div>
          <div className="color-demo">
            <CustomText
              text="Blue Cyan"
              preset="title"
              gradient={{
                colors: ["#3B82F6", "#06B6D4"],
                angle: 45,
              }}
            />
          </div>
          <div className="color-demo">
            <CustomText
              text="Green Yellow"
              preset="title"
              gradient={{
                colors: ["#10B981", "#FBBF24"],
                angle: 45,
              }}
            />
          </div>
          <div className="color-demo">
            <CustomText
              text="Orange Red"
              preset="title"
              gradient={{
                colors: ["#F59E0B", "#EF4444"],
                angle: 45,
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
