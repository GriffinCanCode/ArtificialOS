/**
 * Validation Hook
 * React hook for form and input validation
 */

import { useState, useCallback, useRef } from "react";
import { z } from "zod";
import type { ValidationResult } from "../core/types";

/**
 * Hook for Zod schema validation
 */
export function useValidation<T extends z.ZodType>(schema: T) {
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = useCallback(
    (data: unknown): ValidationResult => {
      const result = schema.safeParse(data);

      if (result.success) {
        setErrors({});
        return { isValid: true, errors: {} };
      }

      const newErrors: Record<string, string> = {};
      result.error.issues.forEach((issue) => {
        const path = issue.path.join(".");
        newErrors[path] = issue.message;
      });

      setErrors(newErrors);
      return { isValid: false, errors: newErrors };
    },
    [schema]
  );

  const validateField = useCallback(
    (fieldName: string, value: unknown): string | null => {
      try {
        if (schema instanceof z.ZodObject) {
          const fieldSchema = schema.shape[fieldName];
          if (fieldSchema) {
            fieldSchema.parse(value);
            setErrors((prev) => {
              const newErrors = { ...prev };
              delete newErrors[fieldName];
              return newErrors;
            });
            return null;
          }
        }
      } catch (error) {
        if (error instanceof z.ZodError) {
          const message = error.issues[0]?.message || "Validation failed";
          setErrors((prev) => ({ ...prev, [fieldName]: message }));
          return message;
        }
      }
      return null;
    },
    [schema]
  );

  const clearErrors = useCallback(() => {
    setErrors({});
  }, []);

  const clearFieldError = useCallback((fieldName: string) => {
    setErrors((prev) => {
      const newErrors = { ...prev };
      delete newErrors[fieldName];
      return newErrors;
    });
  }, []);

  return {
    validate,
    validateField,
    errors,
    clearErrors,
    clearFieldError,
    hasErrors: Object.keys(errors).length > 0,
  };
}

/**
 * Hook for custom validation logic
 */
export function useCustomValidation<T>(
  validator: (value: T) => string | null
) {
  const [error, setError] = useState<string | null>(null);

  const validate = useCallback(
    (value: T): boolean => {
      const result = validator(value);
      setError(result);
      return result === null;
    },
    [validator]
  );

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    validate,
    error,
    clearError,
    isValid: error === null,
  };
}

/**
 * Hook for debounced validation
 */
export function useDebouncedValidation<T extends z.ZodType>(
  schema: T,
  delay: number = 300
) {
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isValidating, setIsValidating] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  const validate = useCallback(
    (data: unknown): Promise<ValidationResult> => {
      return new Promise((resolve) => {
        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current);
        }

        setIsValidating(true);

        timeoutRef.current = setTimeout(() => {
          const result = schema.safeParse(data);

          if (result.success) {
            setErrors({});
            setIsValidating(false);
            resolve({ isValid: true, errors: {} });
          } else {
            const newErrors: Record<string, string> = {};
            result.error.issues.forEach((issue) => {
              const path = issue.path.join(".");
              newErrors[path] = issue.message;
            });

            setErrors(newErrors);
            setIsValidating(false);
            resolve({ isValid: false, errors: newErrors });
          }
        }, delay);
      });
    },
    [schema, delay]
  );

  const clearErrors = useCallback(() => {
    setErrors({});
  }, []);

  return {
    validate,
    errors,
    clearErrors,
    isValidating,
    hasErrors: Object.keys(errors).length > 0,
  };
}
