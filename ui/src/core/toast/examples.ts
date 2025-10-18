/**
 * Toast Examples
 * Comprehensive examples showing different toast usage patterns
 */

import { toast } from "./utils";

// ============================================================================
// Basic Examples
// ============================================================================

export const basicExamples = {
  /**
   * Simple success notification
   */
  simpleSuccess: () => {
    toast.success("Changes saved!");
  },

  /**
   * Error with description
   */
  errorWithDescription: () => {
    toast.error("Upload failed", {
      description: "File size exceeds 10MB limit",
    });
  },

  /**
   * Warning notification
   */
  warning: () => {
    toast.warning("Low disk space available");
  },

  /**
   * Info notification
   */
  info: () => {
    toast.info("New update available", {
      description: "Click to learn more",
    });
  },

  /**
   * Loading state
   */
  loading: () => {
    const id = toast.loading("Processing request...");

    // Simulate async operation
    setTimeout(() => {
      toast.dismiss(id);
      toast.success("Request completed!");
    }, 3000);
  },
};

// ============================================================================
// Advanced Examples
// ============================================================================

export const advancedExamples = {
  /**
   * Toast with action button
   */
  withAction: () => {
    toast.success("Email sent successfully", {
      action: {
        label: "View",
        onClick: () => {
          // Email view action handled by UI interaction
        },
      },
    });
  },

  /**
   * Undo action pattern
   */
  undoPattern: () => {
    let itemDeleted = true;

    toast.undo("Item deleted", () => {
      itemDeleted = false;
      // Item restoration handled by undo system
    });

    // After timeout, permanently delete
    setTimeout(() => {
      if (itemDeleted) {
        // Permanent deletion handled by trash system
      }
    }, 10000);
  },

  /**
   * Progress tracking
   */
  progressTracking: async () => {
    const steps = ["Preparing", "Uploading", "Processing", "Finalizing"];

    for (let i = 0; i < steps.length; i++) {
      const percent = Math.round((i / steps.length) * 100);
      toast.progress(steps[i], percent, {
        id: "progress-demo",
      });

      // Simulate step delay
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    toast.dismiss("progress-demo");
    toast.success("Process completed!");
  },

  /**
   * Promise-based loading
   */
  promiseLoading: () => {
    const fetchData = async () => {
      await new Promise((resolve) => setTimeout(resolve, 2000));
      return { data: "Sample data" };
    };

    toast.promise(fetchData(), {
      loading: "Fetching data...",
      success: "Data loaded successfully!",
      error: "Failed to fetch data",
    });
  },

  /**
   * Custom positioning
   */
  customPosition: () => {
    toast.success("Top right notification", {
      position: "top-right",
    });
  },

  /**
   * Long duration for important messages
   */
  longDuration: () => {
    toast.warning("Important: Server maintenance scheduled", {
      description: "System will be unavailable from 2-4 AM",
      duration: 10000, // 10 seconds
    });
  },
};

// ============================================================================
// Real-World Use Cases
// ============================================================================

export const realWorldExamples = {
  /**
   * File upload with progress
   */
  fileUpload: async (file: File) => {
    const uploadFile = async (file: File) => {
      // Simulate upload with progress
      for (let i = 0; i <= 100; i += 10) {
        toast.progress(`Uploading ${file.name}`, i, {
          id: "file-upload",
        });
        await new Promise((resolve) => setTimeout(resolve, 200));
      }

      return { success: true, url: "/uploads/file.pdf" };
    };

    try {
      await uploadFile(file);
      toast.dismiss("file-upload");
      toast.success("File uploaded successfully!", {
        action: {
          label: "View",
          onClick: () => {}, // File view handled by file manager
        },
      });
    } catch (error) {
      toast.dismiss("file-upload");
      toast.error("Upload failed", {
        description: error instanceof Error ? error.message : "Unknown error",
        action: {
          label: "Retry",
          onClick: () => realWorldExamples.fileUpload(file),
        },
      });
    }
  },

  /**
   * Form submission with validation
   */
  formSubmission: async (formData: Record<string, any>) => {
    const id = toast.loading("Submitting form...");

    try {
      // Simulate API call
      await new Promise((resolve) => setTimeout(resolve, 1500));

      toast.dismiss(id);
      toast.success("Form submitted successfully!", {
        description: "You will receive a confirmation email shortly",
      });
    } catch (error) {
      toast.dismiss(id);
      toast.error("Submission failed", {
        description: "Please check your inputs and try again",
        action: {
          label: "Retry",
          onClick: () => realWorldExamples.formSubmission(formData),
        },
      });
    }
  },

  /**
   * Bulk operations with summary
   */
  bulkDelete: async (itemIds: string[]) => {
    const deletedItems: string[] = [];

    for (let i = 0; i < itemIds.length; i++) {
      const percent = Math.round(((i + 1) / itemIds.length) * 100);
      toast.progress(`Deleting items`, percent, {
        id: "bulk-delete",
      });

      await new Promise((resolve) => setTimeout(resolve, 100));
      deletedItems.push(itemIds[i]);
    }

    toast.dismiss("bulk-delete");
    toast.undo(`Deleted ${itemIds.length} items`, () => {
      // Restore all items
      // Items restoration handled by batch operation system
      toast.success("Items restored");
    });
  },

  /**
   * Network status monitoring
   */
  networkStatus: () => {
    let isOnline = navigator.onLine;

    window.addEventListener("online", () => {
      if (!isOnline) {
        isOnline = true;
        toast.success("Connection restored");
      }
    });

    window.addEventListener("offline", () => {
      if (isOnline) {
        isOnline = false;
        toast.error("Connection lost", {
          description: "Please check your internet connection",
          duration: Infinity, // Don't auto-dismiss
        });
      }
    });
  },

  /**
   * Auto-save indicator
   */
  autoSave: (() => {
    let saveTimeoutId: NodeJS.Timeout;
    let isSaving = false;

    return (_content: string) => {
      // Clear previous timeout
      clearTimeout(saveTimeoutId);

      // Show saving indicator after delay
      saveTimeoutId = setTimeout(async () => {
        if (!isSaving) {
          isSaving = true;
          const id = toast.loading("Saving changes...", {
            duration: Infinity,
          });

          // Simulate save
          await new Promise((resolve) => setTimeout(resolve, 500));

          toast.dismiss(id);
          toast.success("Saved", {
            duration: 2000,
          });
          isSaving = false;
        }
      }, 1000); // Debounce for 1 second
    };
  })(),

  /**
   * Multi-step wizard
   */
  wizardSteps: async () => {
    const steps = [
      { name: "Validate", duration: 1000 },
      { name: "Process", duration: 2000 },
      { name: "Confirm", duration: 500 },
    ];

    for (let i = 0; i < steps.length; i++) {
      const step = steps[i];
      const id = toast.loading(`Step ${i + 1}/${steps.length}: ${step.name}`, {
        description: "Please wait...",
      });

      await new Promise((resolve) => setTimeout(resolve, step.duration));
      toast.dismiss(id);
    }

    toast.success("Wizard completed!", {
      description: "All steps processed successfully",
    });
  },

  /**
   * Copy to clipboard with feedback
   */
  copyToClipboard: async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success("Copied to clipboard!");
    } catch (error) {
      toast.error("Failed to copy", {
        description: "Please try again",
      });
    }
  },

  /**
   * Batch update with error handling
   */
  batchUpdate: async (items: any[]) => {
    const results = {
      success: 0,
      failed: 0,
    };

    toast.loading("Updating items...", { id: "batch-update" });

    for (const _item of items) {
      try {
        // Simulate update
        await new Promise((resolve) => setTimeout(resolve, 100));
        results.success++;
      } catch {
        results.failed++;
      }
    }

    toast.dismiss("batch-update");

    if (results.failed === 0) {
      toast.success(`Updated ${results.success} items successfully`);
    } else {
      toast.warning("Batch update completed with errors", {
        description: `${results.success} succeeded, ${results.failed} failed`,
      });
    }
  },
};

// ============================================================================
// Testing Examples
// ============================================================================

export const testExamples = {
  /**
   * Test all toast types
   */
  testAllTypes: () => {
    const types = [
      () => toast.success("Success toast"),
      () => toast.error("Error toast"),
      () => toast.warning("Warning toast"),
      () => toast.info("Info toast"),
      () => toast.loading("Loading toast"),
    ];

    types.forEach((fn, i) => {
      setTimeout(fn, i * 500);
    });
  },

  /**
   * Test toast positions
   */
  testPositions: () => {
    const positions: Array<any> = [
      "top-left",
      "top-center",
      "top-right",
      "bottom-left",
      "bottom-center",
      "bottom-right",
    ];

    positions.forEach((position, i) => {
      setTimeout(() => {
        toast.info(`Toast at ${position}`, {
          position,
          duration: 3000,
        });
      }, i * 500);
    });
  },

  /**
   * Stress test with many toasts
   */
  stressTest: () => {
    for (let i = 0; i < 10; i++) {
      setTimeout(() => {
        toast.info(`Toast ${i + 1} of 10`);
      }, i * 200);
    }
  },
};
