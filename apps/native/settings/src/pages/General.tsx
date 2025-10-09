/**
 * General Settings Page
 */

import { useState, useEffect } from 'react';
import { SettingRow } from '../components/SettingRow';
import './Page.css';

interface GeneralPageProps {
  executor: any;
  state: any;
}

export function GeneralPage({ executor }: GeneralPageProps) {
  const [theme, setTheme] = useState('dark');
  const [language, setLanguage] = useState('en');
  const [notifications, setNotifications] = useState(true);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);

      // Load theme
      const themeResult = await executor.execute('settings.get', { key: 'general.theme' });
      if (themeResult?.value) setTheme(themeResult.value);

      // Load language
      const langResult = await executor.execute('settings.get', { key: 'general.language' });
      if (langResult?.value) setLanguage(langResult.value);

      // Load notifications
      const notifResult = await executor.execute('settings.get', { key: 'general.notifications' });
      if (notifResult?.value !== undefined) setNotifications(notifResult.value);

    } catch (error) {
      console.error('Failed to load settings:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleThemeChange = async (newTheme: string) => {
    setTheme(newTheme);
    try {
      await executor.execute('settings.set', { key: 'general.theme', value: newTheme });
      await executor.execute('theme.set', { id: newTheme });
    } catch (error) {
      console.error('Failed to update theme:', error);
    }
  };

  const handleLanguageChange = async (newLang: string) => {
    setLanguage(newLang);
    try {
      await executor.execute('settings.set', { key: 'general.language', value: newLang });
    } catch (error) {
      console.error('Failed to update language:', error);
    }
  };

  const handleNotificationsToggle = async () => {
    const newValue = !notifications;
    setNotifications(newValue);
    try {
      await executor.execute('settings.set', { key: 'general.notifications', value: newValue });
    } catch (error) {
      console.error('Failed to update notifications:', error);
    }
  };

  if (loading) {
    return <div className="settings-page loading">Loading...</div>;
  }

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>General Settings</h1>
        <p className="page-description">Configure basic system preferences</p>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Appearance</h2>
        <SettingRow
          label="Theme"
          description="Choose your preferred color scheme"
        >
          <select value={theme} onChange={(e) => handleThemeChange(e.target.value)}>
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="high-contrast">High Contrast</option>
          </select>
        </SettingRow>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Localization</h2>
        <SettingRow
          label="Language"
          description="Select your preferred language"
        >
          <select value={language} onChange={(e) => handleLanguageChange(e.target.value)}>
            <option value="en">English</option>
            <option value="es">Español</option>
            <option value="fr">Français</option>
            <option value="de">Deutsch</option>
            <option value="zh">中文</option>
          </select>
        </SettingRow>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Notifications</h2>
        <SettingRow
          label="Enable Notifications"
          description="Show system and app notifications"
        >
          <div
            className={`toggle-switch ${notifications ? 'active' : ''}`}
            onClick={handleNotificationsToggle}
          />
        </SettingRow>
      </div>
    </div>
  );
}

