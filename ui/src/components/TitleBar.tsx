/**
 * Custom Title Bar Component
 * Provides window controls for frameless Electron window
 */

import React from 'react';
import './TitleBar.css';

const TitleBar: React.FC = () => {
  const handleMinimize = () => {
    if (window.electron) {
      window.electron.minimize();
    }
  };

  const handleMaximize = () => {
    if (window.electron) {
      window.electron.maximize();
    }
  };

  const handleClose = () => {
    if (window.electron) {
      window.electron.close();
    }
  };

  return (
    <div className="title-bar">
      <div className="title-bar-drag">
        <span className="title">🤖 AI-Powered OS</span>
      </div>
      <div className="window-controls">
        <button className="control-btn minimize" onClick={handleMinimize}>
          ─
        </button>
        <button className="control-btn maximize" onClick={handleMaximize}>
          □
        </button>
        <button className="control-btn close" onClick={handleClose}>
          ✕
        </button>
      </div>
    </div>
  );
};

export default TitleBar;

