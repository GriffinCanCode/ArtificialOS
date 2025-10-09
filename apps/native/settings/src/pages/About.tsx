/**
 * About Page
 */

import './Page.css';

interface AboutPageProps {
  executor: any;
  state: any;
}

export function AboutPage(_props: AboutPageProps) {
  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>About</h1>
        <p className="page-description">Information about AI-OS</p>
      </div>

      <div className="settings-section">
        <div className="about-content">
          <div className="about-logo">⚙️</div>
          <h2 className="about-title">AI-OS</h2>
          <p className="about-version">Version 1.0.0</p>
          <p className="about-description">
            A production-grade operating system kernel and runtime for AI-powered applications.
          </p>
        </div>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Features</h2>
        <ul className="feature-list">
          <li>Custom Rust kernel with async syscalls</li>
          <li>Multi-app support with process isolation</li>
          <li>Dynamic UI generation via LLM</li>
          <li>Native TypeScript/React apps</li>
          <li>Real-time system monitoring</li>
          <li>Fine-grained permission system</li>
          <li>Distributed tracing and observability</li>
          <li>Session management and persistence</li>
        </ul>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Architecture</h2>
        <div className="architecture-grid">
          <div className="arch-item">
            <div className="arch-title">Kernel (Rust)</div>
            <div className="arch-desc">Process management, IPC, filesystem, networking</div>
          </div>
          <div className="arch-item">
            <div className="arch-title">Backend (Go)</div>
            <div className="arch-desc">HTTP API, service providers, gRPC clients</div>
          </div>
          <div className="arch-item">
            <div className="arch-title">AI Service (Python)</div>
            <div className="arch-desc">LLM integration, UI generation, agent orchestration</div>
          </div>
          <div className="arch-item">
            <div className="arch-title">Frontend (TypeScript/React)</div>
            <div className="arch-desc">Window management, dynamic rendering, native apps</div>
          </div>
        </div>
      </div>

      <div className="settings-section">
        <h2 className="section-title">License</h2>
        <p>MIT License © 2025 AI-OS Contributors</p>
      </div>
    </div>
  );
}

