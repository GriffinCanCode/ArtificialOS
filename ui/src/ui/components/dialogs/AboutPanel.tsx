/**
 * About Panel Component
 * Modern, clean system information panel
 */

import React, { useCallback, useEffect } from "react";
import { X, Github, Book, Sparkles, Cpu, Zap } from "lucide-react";
import { useLogger } from "../../../core/utils/monitoring/useLogger";
import { Tooltip } from "../../../features/floating";
import "./AboutPanel.css";

interface AboutPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AboutPanel: React.FC<AboutPanelProps> = ({ isOpen, onClose }) => {
  const log = useLogger("AboutPanel");

  // Close on ESC key
  useEffect(() => {
    if (!isOpen) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [isOpen, onClose]);

  const handleOverlayClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose();
      }
    },
    [onClose]
  );

  if (!isOpen) return null;

  return (
    <div className="about-overlay" onClick={handleOverlayClick}>
      <div className="about-panel">
        {/* Header */}
        <div className="about-header">
          <div className="about-logo">
            <Sparkles size={24} className="logo-icon" />
            <div className="about-title">
              <h2>AgentOS</h2>
              <span className="about-tagline">AI-Powered Operating System</span>
            </div>
          </div>
          <Tooltip content="Close" delay={500}>
            <button className="about-close" onClick={onClose} aria-label="Close">
              <X size={18} />
            </button>
          </Tooltip>
        </div>

        {/* Content Grid */}
        <div className="about-content">
          {/* Version Card */}
          <div className="about-card version-card">
            <div className="card-label">Version</div>
            <div className="card-value">1.0.0-alpha</div>
            <div className="card-sublabel">Build 2025.10</div>
          </div>

          {/* System Status */}
          <div className="about-card status-card">
            <div className="card-label">System Status</div>
            <div className="status-indicators">
              <div className="status-item">
                <div className="status-dot active"></div>
                <span>Kernel</span>
              </div>
              <div className="status-item">
                <div className="status-dot active"></div>
                <span>Backend</span>
              </div>
              <div className="status-item">
                <div className="status-dot active"></div>
                <span>AI Service</span>
              </div>
            </div>
          </div>

          {/* Components Info */}
          <div className="about-card components-card">
            <div className="card-label">Architecture</div>
            <div className="component-list">
              <div className="component-item">
                <Cpu size={16} className="component-icon" />
                <span>Rust Kernel</span>
              </div>
              <div className="component-item">
                <Zap size={16} className="component-icon" />
                <span>Go Backend</span>
              </div>
              <div className="component-item">
                <Sparkles size={16} className="component-icon" />
                <span>Python AI</span>
              </div>
            </div>
          </div>

          {/* Quick Links */}
          <div className="about-card links-card">
            <div className="card-label">Resources</div>
            <div className="quick-links">
              <a
                href="https://github.com/GriffinCanCode/AgentOS"
                target="_blank"
                rel="noopener noreferrer"
                className="quick-link"
              >
                <Github size={16} />
                <span>GitHub</span>
              </a>
              <a
                href="#"
                onClick={(e) => {
                  e.preventDefault();
                  log.info("Documentation link clicked");
                }}
                className="quick-link"
              >
                <Book size={16} />
                <span>Docs</span>
              </a>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="about-footer">
          <p>Built with Rust, Go, Python & TypeScript</p>
          <p className="copyright">© 2025 AgentOS. Open Source.</p>
        </div>
      </div>
    </div>
  );
};

