/**
 * Component Validation Tests
 * Tests for Zod schema validation system
 */

import { describe, it, expect } from "vitest";
import { z } from "zod";
import {
  validateComponentProps,
  safeParseProps,
  formatValidationErrors,
} from "../../../src/features/dynamics/core/validation";

// Import some schemas to test
import {
  buttonSchema,
  inputSchema,
  textSchema,
} from "../../../src/features/dynamics/schemas/primitives";

import { imageSchema, videoSchema } from "../../../src/features/dynamics/schemas/media";

describe("Component Validation", () => {
  describe("validateComponentProps", () => {
    it("validates valid button props", () => {
      const props = {
        text: "Click me",
        variant: "primary",
        size: "medium",
      };

      const result = validateComponentProps(props, buttonSchema, "button");

      expect(result.success).toBe(true);
      expect(result.data).toEqual(props);
    });

    it("validates valid input props", () => {
      const props = {
        type: "email",
        placeholder: "Enter email",
        variant: "filled",
      };

      const result = validateComponentProps(props, inputSchema, "input");

      expect(result.success).toBe(true);
      expect(result.data).toEqual(props);
    });

    it("fails validation for invalid button variant", () => {
      const props = {
        text: "Click me",
        variant: "invalid-variant",
      };

      const result = validateComponentProps(props, buttonSchema, "button");

      expect(result.success).toBe(false);
      expect(result.errors).toBeDefined();
    });

    it("fails validation for invalid image URL", () => {
      const props = {
        src: "not-a-url",
        alt: "Test image",
      };

      const result = validateComponentProps(props, imageSchema, "image");

      expect(result.success).toBe(false);
      expect(result.errors).toBeDefined();
    });

    it("validates valid image URL", () => {
      const props = {
        src: "https://example.com/image.jpg",
        alt: "Test image",
        fit: "cover",
      };

      const result = validateComponentProps(props, imageSchema, "image");

      expect(result.success).toBe(true);
      expect(result.data).toEqual(props);
    });

    it("allows optional props to be omitted", () => {
      const props = {
        text: "Button",
      };

      const result = validateComponentProps(props, buttonSchema, "button");

      expect(result.success).toBe(true);
      expect(result.data).toEqual(props);
    });

    it("validates required text content", () => {
      const propsValid = {
        content: "Hello World",
      };

      const resultValid = validateComponentProps(propsValid, textSchema, "text");
      expect(resultValid.success).toBe(true);

      const propsInvalid = {
        variant: "h1",
      };

      const resultInvalid = validateComponentProps(propsInvalid, textSchema, "text");
      expect(resultInvalid.success).toBe(false);
    });

    it("validates video props with controls", () => {
      const props = {
        src: "https://example.com/video.mp4",
        controls: true,
        autoPlay: false,
        loop: true,
      };

      const result = validateComponentProps(props, videoSchema, "video");

      expect(result.success).toBe(true);
      expect(result.data).toEqual(props);
    });
  });

  describe("safeParseProps", () => {
    it("returns validated data on success", () => {
      const props = {
        text: "Click me",
        variant: "primary",
      };

      const result = safeParseProps(props, buttonSchema);

      expect(result).toEqual(props);
    });

    it("returns original props on validation failure", () => {
      const props = {
        text: "Click me",
        variant: "invalid-variant",
      };

      const result = safeParseProps(props, buttonSchema);

      expect(result).toEqual(props);
    });

    it("handles missing required fields gracefully", () => {
      const props = {
        variant: "h1",
        // Missing required 'content' field
      };

      const result = safeParseProps(props, textSchema);

      // Should return original props even though validation failed
      expect(result).toEqual(props);
    });
  });

  describe("formatValidationErrors", () => {
    it("formats validation errors with paths", () => {
      const schema = z.object({
        name: z.string().min(3),
        age: z.number().min(0),
      });

      try {
        schema.parse({ name: "ab", age: -1 });
      } catch (error) {
        if (error instanceof z.ZodError) {
          const formatted = formatValidationErrors(error);
          expect(formatted).toHaveLength(2);
          expect(formatted[0]).toContain("name");
          expect(formatted[1]).toContain("age");
        }
      }
    });

    it("formats errors without paths", () => {
      const schema = z.string().url();

      try {
        schema.parse("not-a-url");
      } catch (error) {
        if (error instanceof z.ZodError) {
          const formatted = formatValidationErrors(error);
          expect(formatted).toHaveLength(1);
          expect(formatted[0]).toContain("Invalid url");
        }
      }
    });
  });

  describe("Schema Coverage", () => {
    it("has schemas for all primitive components", () => {
      expect(buttonSchema).toBeDefined();
      expect(inputSchema).toBeDefined();
      expect(textSchema).toBeDefined();
    });

    it("has schemas for all media components", () => {
      expect(imageSchema).toBeDefined();
      expect(videoSchema).toBeDefined();
    });

    it("validates style prop as optional record", () => {
      const props = {
        text: "Button",
        style: {
          color: "red",
          fontSize: "16px",
        },
      };

      const result = validateComponentProps(props, buttonSchema, "button");

      expect(result.success).toBe(true);
      expect(result.data?.style).toEqual(props.style);
    });
  });
});
