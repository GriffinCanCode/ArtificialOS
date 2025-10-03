/**
 * Save Session Dialog
 * Form dialog for saving sessions using React Hook Form
 */

import React, { useCallback } from "react";
import { useForm } from "react-hook-form";
import { Modal } from "./Modal";
import "./SaveSessionDialog.css";

interface SaveSessionFormData {
  name: string;
  description?: string;
}

interface SaveSessionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (data: SaveSessionFormData) => Promise<void>;
  isLoading?: boolean;
}

export const SaveSessionDialog: React.FC<SaveSessionDialogProps> = React.memo(({
  isOpen,
  onClose,
  onSave,
  isLoading = false,
}) => {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
    reset,
  } = useForm<SaveSessionFormData>({
    defaultValues: {
      name: "",
      description: "",
    },
  });

  const onSubmit = useCallback(async (data: SaveSessionFormData) => {
    try {
      await onSave(data);
      reset();
      onClose();
    } catch (error) {
      // Error handling is done in parent component
      console.error("Failed to save session:", error);
    }
  }, [onSave, reset, onClose]);

  const handleClose = useCallback(() => {
    reset();
    onClose();
  }, [reset, onClose]);

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Save Session">
      <form onSubmit={handleSubmit(onSubmit)} className="save-session-form">
        <div className="form-group">
          <label htmlFor="session-name" className="form-label">
            Session Name <span className="required">*</span>
          </label>
          <input
            id="session-name"
            type="text"
            className={`form-input ${errors.name ? "error" : ""}`}
            placeholder="e.g., Work Session"
            {...register("name", {
              required: "Session name is required",
              minLength: {
                value: 2,
                message: "Name must be at least 2 characters",
              },
              maxLength: {
                value: 50,
                message: "Name must be less than 50 characters",
              },
            })}
            autoFocus
          />
          {errors.name && (
            <span className="form-error">{errors.name.message}</span>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="session-description" className="form-label">
            Description <span className="optional">(optional)</span>
          </label>
          <textarea
            id="session-description"
            className="form-textarea"
            placeholder="What were you working on?"
            rows={3}
            {...register("description", {
              maxLength: {
                value: 200,
                message: "Description must be less than 200 characters",
              },
            })}
          />
          {errors.description && (
            <span className="form-error">{errors.description.message}</span>
          )}
        </div>

        <div className="form-actions">
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleClose}
            disabled={isSubmitting || isLoading}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="btn btn-primary"
            disabled={isSubmitting || isLoading}
          >
            {isSubmitting || isLoading ? "Saving..." : "Save Session"}
          </button>
        </div>
      </form>
    </Modal>
  );
});

SaveSessionDialog.displayName = 'SaveSessionDialog';

