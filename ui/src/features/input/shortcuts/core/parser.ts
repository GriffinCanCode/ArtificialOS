/**
 * Shortcut Parser
 * Parse and normalize keyboard shortcut sequences
 */

import { normalizeKey, resolveMod, detectPlatform } from "./platform";
import type { Platform } from "./types";

// ============================================================================
// Parser Types
// ============================================================================

interface ParsedSequence {
  modifiers: string[];
  key: string;
  original: string;
  normalized: string;
}

// ============================================================================
// Parser Functions
// ============================================================================

/**
 * Parse shortcut sequence into components
 */
export function parseSequence(
  sequence: string,
  platform?: Platform
): ParsedSequence {
  const p = platform || detectPlatform();
  
  // Split by + or space
  const parts = sequence
    .split(/[\+\s]+/)
    .map((part) => part.trim())
    .filter(Boolean);
  
  if (parts.length === 0) {
    throw new Error(`Invalid shortcut sequence: ${sequence}`);
  }
  
  // Last part is the key, rest are modifiers
  const key = normalizeKey(parts[parts.length - 1]);
  const modifiers = parts
    .slice(0, -1)
    .map((mod) => resolveMod(normalizeKey(mod), p))
    .sort(); // Sort for consistent comparison
  
  // Build normalized sequence
  const normalized = modifiers.length > 0 
    ? `${modifiers.join("+")}+${key}`
    : key;
  
  return {
    modifiers,
    key,
    original: sequence,
    normalized,
  };
}

/**
 * Validate shortcut sequence
 */
export function validateSequence(sequence: string): {
  valid: boolean;
  error?: string;
} {
  try {
    const parsed = parseSequence(sequence);
    
    // Check for empty key
    if (!parsed.key) {
      return { valid: false, error: "Key cannot be empty" };
    }
    
    // Check for duplicate modifiers
    const uniqueMods = new Set(parsed.modifiers);
    if (uniqueMods.size !== parsed.modifiers.length) {
      return { valid: false, error: "Duplicate modifiers detected" };
    }
    
    // Check for modifier-only sequence
    if (parsed.modifiers.length === 0 && isModifierKey(parsed.key)) {
      return { valid: false, error: "Cannot use modifier as main key" };
    }
    
    return { valid: true };
  } catch (error) {
    return {
      valid: false,
      error: error instanceof Error ? error.message : "Invalid sequence",
    };
  }
}

/**
 * Check if key is a modifier
 */
export function isModifierKey(key: string): boolean {
  const modifiers = ["Control", "Meta", "Alt", "Shift", "cmd", "ctrl", "option", "command"];
  return modifiers.includes(key);
}

/**
 * Normalize sequence for comparison
 */
export function normalizeSequence(
  sequence: string,
  platform?: Platform
): string {
  try {
    const parsed = parseSequence(sequence, platform);
    return parsed.normalized;
  } catch {
    return sequence;
  }
}

/**
 * Compare two sequences for equality
 */
export function sequencesEqual(
  seq1: string,
  seq2: string,
  platform?: Platform
): boolean {
  const norm1 = normalizeSequence(seq1, platform);
  const norm2 = normalizeSequence(seq2, platform);
  return norm1 === norm2;
}

/**
 * Extract modifiers from keyboard event
 */
export function extractModifiers(event: KeyboardEvent): string[] {
  const modifiers: string[] = [];
  
  if (event.ctrlKey) modifiers.push("Control");
  if (event.metaKey) modifiers.push("Meta");
  if (event.altKey) modifiers.push("Alt");
  if (event.shiftKey) modifiers.push("Shift");
  
  return modifiers.sort();
}

/**
 * Build sequence from keyboard event
 */
export function eventToSequence(event: KeyboardEvent): string {
  const modifiers = extractModifiers(event);
  const key = normalizeKey(event.key);
  
  if (modifiers.length === 0) {
    return key;
  }
  
  return `${modifiers.join("+")}+${key}`;
}

/**
 * Check if event matches sequence
 */
export function matchesSequence(
  event: KeyboardEvent,
  sequence: string,
  platform?: Platform
): boolean {
  const eventSeq = eventToSequence(event);
  return sequencesEqual(eventSeq, sequence, platform);
}

// ============================================================================
// Advanced Parsing
// ============================================================================

/**
 * Parse multiple sequences (for sequence chains)
 */
export function parseSequenceChain(
  chain: string,
  platform?: Platform
): ParsedSequence[] {
  // Split by space for sequence chains (e.g., "g d" = press g then d)
  const sequences = chain.split(/\s+/).filter(Boolean);
  return sequences.map((seq) => parseSequence(seq, platform));
}

/**
 * Check if sequence is a chord (multiple keys pressed together)
 */
export function isChord(sequence: string): boolean {
  return sequence.includes("+");
}

/**
 * Check if sequence is a chain (sequential key presses)
 */
export function isChain(sequence: string): boolean {
  const parts = sequence.split(/\s+/).filter(Boolean);
  return parts.length > 1;
}

/**
 * Get complexity score of sequence (for conflict resolution)
 */
export function getComplexity(sequence: string): number {
  const parsed = parseSequence(sequence);
  
  // Base complexity: number of modifiers + 1 for the key
  let complexity = parsed.modifiers.length + 1;
  
  // Add complexity for chains
  if (isChain(sequence)) {
    const chain = parseSequenceChain(sequence);
    complexity += chain.length * 2;
  }
  
  return complexity;
}

