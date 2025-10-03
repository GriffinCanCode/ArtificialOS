/**
 * Save App Dialog
 * Form dialog for saving apps to registry using React Hook Form
 */

import React, { useCallback } from "react";
import { useForm } from "react-hook-form";
import { Modal } from "./Modal";
import "./SaveAppDialog.css";

interface SaveAppFormData {
  description: string;
  category: string;
  icon: string;
  tags?: string;
}

interface SaveAppDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (data: Omit<SaveAppFormData, "tags"> & { tags: string[] }) => Promise<void>;
  isLoading?: boolean;
}

const CATEGORIES = [
  { value: "productivity", label: "Productivity" },
  { value: "utilities", label: "Utilities" },
  { value: "games", label: "Games" },
  { value: "creative", label: "Creative" },
  { value: "general", label: "General" },
];

const SUGGESTED_ICONS = ["📦", "⚡", "🎯", "🎨", "🎮", "🛠️", "📊", "🔧"];

export const SaveAppDialog: React.FC<SaveAppDialogProps> = React.memo(({
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
    watch,
    setValue,
  } = useForm<SaveAppFormData>({
    defaultValues: {
      description: "",
      category: "general",
      icon: "📦",
      tags: "",
    },
  });

  const currentIcon = watch("icon");

  const onSubmit = useCallback(async (data: SaveAppFormData) => {
    try {
      // Convert tags string to array
      const tags = data.tags
        ? data.tags.split(",").map((tag) => tag.trim()).filter(Boolean)
        : [];

      await onSave({
        description: data.description,
        category: data.category,
        icon: data.icon,
        tags,
      });
      reset();
      onClose();
    } catch (error) {
      // Error handling is done in parent component
      console.error("Failed to save app:", error);
    }
  }, [onSave, reset, onClose]);

  const handleClose = useCallback(() => {
    reset();
    onClose();
  }, [reset, onClose]);

  const handleIconSelect = useCallback((icon: string) => {
    setValue("icon", icon);
  }, [setValue]);

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Save App to Registry">
      <form onSubmit={handleSubmit(onSubmit)} className="save-app-form">
        <div className="form-group">
          <label htmlFor="app-description" className="form-label">
            Description <span className="required">*</span>
          </label>
          <textarea
            id="app-description"
            className={`form-textarea ${errors.description ? "error" : ""}`}
            placeholder="Describe what this app does..."
            rows={3}
            {...register("description", {
              required: "Description is required",
              minLength: {
                value: 10,
                message: "Description must be at least 10 characters",
              },
              maxLength: {
                value: 200,
                message: "Description must be less than 200 characters",
              },
            })}
            autoFocus
          />
          {errors.description && (
            <span className="form-error">{errors.description.message}</span>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="app-category" className="form-label">
            Category <span className="required">*</span>
          </label>
          <select
            id="app-category"
            className={`form-select ${errors.category ? "error" : ""}`}
            {...register("category", { required: "Category is required" })}
          >
            {CATEGORIES.map((cat) => (
              <option key={cat.value} value={cat.value}>
                {cat.label}
              </option>
            ))}
          </select>
          {errors.category && (
            <span className="form-error">{errors.category.message}</span>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="app-icon" className="form-label">
            Icon <span className="required">*</span>
          </label>
          <div className="icon-selector">
            <input
              id="app-icon"
              type="text"
              className={`form-input icon-input ${errors.icon ? "error" : ""}`}
              placeholder="📦"
              maxLength={4}
              {...register("icon", {
                required: "Icon is required",
              })}
            />
            <div className="icon-preview">{currentIcon}</div>
          </div>
          <div className="icon-suggestions">
            {SUGGESTED_ICONS.map((icon) => (
              <button
                key={icon}
                type="button"
                className={`icon-option ${currentIcon === icon ? "active" : ""}`}
                onClick={() => handleIconSelect(icon)}
              >
                {icon}
              </button>
            ))}
          </div>
          {errors.icon && (
            <span className="form-error">{errors.icon.message}</span>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="app-tags" className="form-label">
            Tags <span className="optional">(optional, comma-separated)</span>
          </label>
          <input
            id="app-tags"
            type="text"
            className="form-input"
            placeholder="e.g., calculator, math, tools"
            {...register("tags")}
          />
          <span className="form-hint">
            Separate tags with commas to help users find your app
          </span>
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
            {isSubmitting || isLoading ? "Saving..." : "Save to Registry"}
          </button>
        </div>
      </form>
    </Modal>
  );
});

SaveAppDialog.displayName = 'SaveAppDialog';

