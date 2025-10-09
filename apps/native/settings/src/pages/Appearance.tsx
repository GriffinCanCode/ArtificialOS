/**
 * Appearance Settings Page
 */

import { useState, useEffect } from 'react';
import { SettingRow } from '../components/SettingRow';
import './Page.css';

interface AppearancePageProps {
  executor: any;
  state: any;
}

export function AppearancePage({ executor }: AppearancePageProps) {
  const [fontSize, setFontSize] = useState(14);
  const [fontFamily, setFontFamily] = useState('Inter');
  const [accentColor, setAccentColor] = useState('#3b82f6');
  const [themes, setThemes] = useState<any[]>([]);
  const [currentTheme, setCurrentTheme] = useState<any>(null);

  useEffect(() => {
    loadSettings();
    loadThemes();
  }, []);

  const loadSettings = async () => {
    try {
      const fontSizeResult = await executor.execute('settings.get', { key: 'appearance.font_size' });
      if (fontSizeResult?.value) setFontSize(fontSizeResult.value);

      const fontFamilyResult = await executor.execute('settings.get', { key: 'appearance.font_family' });
      if (fontFamilyResult?.value) setFontFamily(fontFamilyResult.value);

      const accentResult = await executor.execute('settings.get', { key: 'appearance.accent_color' });
      if (accentResult?.value) setAccentColor(accentResult.value);
    } catch (error) {
      console.error('Failed to load appearance settings:', error);
    }
  };

  const loadThemes = async () => {
    try {
      const result = await executor.execute('theme.list', {});
      if (result?.themes) setThemes(result.themes);

      const current = await executor.execute('theme.current', {});
      if (current) setCurrentTheme(current);
    } catch (error) {
      console.error('Failed to load themes:', error);
    }
  };

  const handleFontSizeChange = async (value: number) => {
    setFontSize(value);
    try {
      await executor.execute('settings.set', { key: 'appearance.font_size', value });
    } catch (error) {
      console.error('Failed to update font size:', error);
    }
  };

  const handleFontFamilyChange = async (value: string) => {
    setFontFamily(value);
    try {
      await executor.execute('settings.set', { key: 'appearance.font_family', value });
    } catch (error) {
      console.error('Failed to update font family:', error);
    }
  };

  const handleAccentColorChange = async (value: string) => {
    setAccentColor(value);
    try {
      await executor.execute('settings.set', { key: 'appearance.accent_color', value });
    } catch (error) {
      console.error('Failed to update accent color:', error);
    }
  };

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>Appearance Settings</h1>
        <p className="page-description">Customize the look and feel of your interface</p>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Typography</h2>
        <SettingRow
          label="Font Size"
          description="Base font size in pixels"
        >
          <input
            type="number"
            value={fontSize}
            onChange={(e) => handleFontSizeChange(Number(e.target.value))}
            min="10"
            max="24"
            style={{ width: '80px' }}
          />
        </SettingRow>
        <SettingRow
          label="Font Family"
          description="Default font for the interface"
        >
          <select value={fontFamily} onChange={(e) => handleFontFamilyChange(e.target.value)}>
            <option value="Inter">Inter</option>
            <option value="system-ui">System UI</option>
            <option value="Roboto">Roboto</option>
            <option value="sans-serif">Sans Serif</option>
          </select>
        </SettingRow>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Colors</h2>
        <SettingRow
          label="Accent Color"
          description="Primary accent color for UI elements"
        >
          <input
            type="color"
            value={accentColor}
            onChange={(e) => handleAccentColorChange(e.target.value)}
          />
        </SettingRow>
      </div>

      {themes.length > 0 && (
        <div className="settings-section">
          <h2 className="section-title">Available Themes</h2>
          <div className="theme-grid">
            {themes.map((theme) => (
              <div
                key={theme.id}
                className={`theme-card ${currentTheme?.id === theme.id ? 'active' : ''}`}
                onClick={async () => {
                  try {
                    await executor.execute('theme.set', { id: theme.id });
                    setCurrentTheme(theme);
                  } catch (error) {
                    console.error('Failed to set theme:', error);
                  }
                }}
              >
                <div className="theme-name">{theme.name}</div>
                <div className="theme-type">{theme.type}</div>
                {theme.description && (
                  <div className="theme-description">{theme.description}</div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

