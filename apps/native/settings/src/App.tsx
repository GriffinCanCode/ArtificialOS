/**
 * Settings App
 * System configuration and preferences management
 */

import { useState, useEffect } from 'react';
import type { NativeAppProps } from './sdk';
import { Sidebar } from './components/Sidebar';
import { GeneralPage } from './pages/General';
import { SystemPage } from './pages/System';
import { AppearancePage } from './pages/Appearance';
import { NetworkPage } from './pages/Network';
import { PermissionsPage } from './pages/Permissions';
import { StoragePage } from './pages/Storage';
import { DeveloperPage } from './pages/Developer';
import { AboutPage } from './pages/About';
import './styles/App.css';

export type Page = 'general' | 'system' | 'appearance' | 'network' | 'permissions' | 'storage' | 'developer' | 'about';

export default function SettingsApp({ context }: NativeAppProps) {
  const { window: win, executor, state } = context;
  const [activePage, setActivePage] = useState<Page>('general');
  const [searchQuery, setSearchQuery] = useState('');

  // Initialize
  useEffect(() => {
    win.setTitle('⚙️ Settings');
    win.maximize();
  }, [win]);

  // Render active page
  const renderPage = () => {
    const pageProps = { executor, state };

    switch (activePage) {
      case 'general':
        return <GeneralPage {...pageProps} />;
      case 'system':
        return <SystemPage {...pageProps} />;
      case 'appearance':
        return <AppearancePage {...pageProps} />;
      case 'network':
        return <NetworkPage {...pageProps} />;
      case 'permissions':
        return <PermissionsPage {...pageProps} />;
      case 'storage':
        return <StoragePage {...pageProps} />;
      case 'developer':
        return <DeveloperPage {...pageProps} />;
      case 'about':
        return <AboutPage {...pageProps} />;
      default:
        return <GeneralPage {...pageProps} />;
    }
  };

  return (
    <div className="settings-app">
      <Sidebar
        activePage={activePage}
        onNavigate={setActivePage}
        searchQuery={searchQuery}
        onSearch={setSearchQuery}
      />
      <div className="settings-content">
        {renderPage()}
      </div>
    </div>
  );
}
