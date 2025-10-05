/**
 * Input Validators
 * Pure validation functions for common input patterns
 */

// ============================================================================
// Text Validators
// ============================================================================

export function isEmpty(value: string): boolean {
  return value.trim().length === 0;
}

export function isEmail(value: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(value);
}

export function isUrl(value: string): boolean {
  try {
    new URL(value);
    return true;
  } catch {
    return false;
  }
}

export function isPhone(value: string): boolean {
  const phoneRegex = /^\+?[\d\s\-()]+$/;
  return phoneRegex.test(value);
}

export function isAlphanumeric(value: string): boolean {
  return /^[a-zA-Z0-9]+$/.test(value);
}

export function isSlug(value: string): boolean {
  return /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(value);
}

// ============================================================================
// Number Validators
// ============================================================================

export function isNumber(value: any): boolean {
  return typeof value === "number" && !isNaN(value) && isFinite(value);
}

export function isInteger(value: number): boolean {
  return Number.isInteger(value);
}

export function isInRange(value: number, min: number, max: number): boolean {
  return value >= min && value <= max;
}

export function isPositive(value: number): boolean {
  return value > 0;
}

export function isNonNegative(value: number): boolean {
  return value >= 0;
}

// ============================================================================
// Length Validators
// ============================================================================

export function hasMinLength(value: string, min: number): boolean {
  return value.length >= min;
}

export function hasMaxLength(value: string, max: number): boolean {
  return value.length <= max;
}

export function isLengthBetween(value: string, min: number, max: number): boolean {
  return value.length >= min && value.length <= max;
}

// ============================================================================
// Pattern Validators
// ============================================================================

export function matchesPattern(value: string, pattern: RegExp): boolean {
  return pattern.test(value);
}

export function containsUppercase(value: string): boolean {
  return /[A-Z]/.test(value);
}

export function containsLowercase(value: string): boolean {
  return /[a-z]/.test(value);
}

export function containsNumber(value: string): boolean {
  return /[0-9]/.test(value);
}

export function containsSpecialChar(value: string): boolean {
  return /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(value);
}

// ============================================================================
// File Validators
// ============================================================================

export function isValidFileName(value: string): boolean {
  return /^[^<>:"/\\|?*]+$/.test(value) && value.length > 0 && value.length <= 255;
}

export function isValidPath(value: string): boolean {
  return /^[^\0]+$/.test(value) && value.length > 0;
}

export function hasFileExtension(value: string, extensions: string[]): boolean {
  const ext = value.toLowerCase().split(".").pop();
  return ext ? extensions.includes(ext) : false;
}

// ============================================================================
// Custom Validators
// ============================================================================

export function createMinValidator(min: number) {
  return (value: number) => value >= min;
}

export function createMaxValidator(max: number) {
  return (value: number) => value <= max;
}

export function createPatternValidator(pattern: RegExp) {
  return (value: string) => pattern.test(value);
}

export function createLengthValidator(min: number, max?: number) {
  return (value: string) => {
    if (value.length < min) return false;
    if (max !== undefined && value.length > max) return false;
    return true;
  };
}

// ============================================================================
// Composite Validators
// ============================================================================

export function validateAll(value: any, validators: Array<(v: any) => boolean>): boolean {
  return validators.every((validator) => validator(value));
}

export function validateAny(value: any, validators: Array<(v: any) => boolean>): boolean {
  return validators.some((validator) => validator(value));
}
