/**
 * Confirm Dialog Component
 * Modal dialog for confirmations (replaces confirm())
 */

import { useEffect } from 'react';

export interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: 'default' | 'danger';
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  title,
  message,
  confirmText = 'OK',
  cancelText = 'Cancel',
  variant = 'default',
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
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

  const handleConfirm = () => {
    onConfirm();
  };

  return (
    <div className="dialog-overlay">
      <div className="dialog">
        <div className="dialog-header">
          <h3 className="dialog-title">{title}</h3>
        </div>

        <div className="dialog-body">
          <p className="dialog-message">{message}</p>
        </div>

        <div className="dialog-footer">
          <button
            className="dialog-button dialog-button-secondary"
            onClick={onCancel}
          >
            {cancelText}
          </button>
          <button
            className={`dialog-button ${
              variant === 'danger'
                ? 'dialog-button-danger'
                : 'dialog-button-primary'
            }`}
            onClick={handleConfirm}
          >
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
}
