/**
 * Input Dialog Component
 * Modal dialog for text input (replaces prompt())
 */

import { useEffect, useRef, useState, KeyboardEvent } from 'react';

export interface InputDialogProps {
  title: string;
  label: string;
  defaultValue?: string;
  placeholder?: string;
  onConfirm: (value: string) => void;
  onCancel: () => void;
}

export function InputDialog({
  title,
  label,
  defaultValue = '',
  placeholder = '',
  onConfirm,
  onCancel,
}: InputDialogProps) {
  const [value, setValue] = useState(defaultValue);
  const inputRef = useRef<HTMLInputElement>(null);

  // Auto-focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  // Handle escape key
  useEffect(() => {
    const handleEscape = (e: globalThis.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onCancel();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [onCancel]);

  const handleSubmit = () => {
    if (value.trim()) {
      onConfirm(value.trim());
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
    }
  };

  return (
    <div className="dialog-overlay">
      <div className="dialog">
        <div className="dialog-header">
          <h3 className="dialog-title">{title}</h3>
        </div>

        <div className="dialog-body">
          <label className="dialog-label">{label}</label>
          <input
            ref={inputRef}
            type="text"
            className="dialog-input"
            value={value}
            placeholder={placeholder}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={handleKeyDown}
          />
        </div>

        <div className="dialog-footer">
          <button
            className="dialog-button dialog-button-secondary"
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            className="dialog-button dialog-button-primary"
            onClick={handleSubmit}
            disabled={!value.trim()}
          >
            OK
          </button>
        </div>
      </div>
    </div>
  );
}
