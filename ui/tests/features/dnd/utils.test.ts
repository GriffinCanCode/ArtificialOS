/**
 * Drag & Drop Utilities Tests
 */

import { describe, it, expect } from "vitest";
import {
  validateFileType,
  validateFileSize,
  validateFile,
  processFiles,
  arrayMove,
  arrayInsert,
  arrayRemove,
  formatFileSize,
  getFileExtension,
  isImageFile,
} from "../../../src/features/dnd/core/utils";

describe("validateFileType", () => {
  it("should accept any type when no accept list provided", () => {
    const file = new File(["content"], "test.txt", { type: "text/plain" });
    expect(validateFileType(file, undefined)).toBe(true);
  });

  it("should validate exact MIME type", () => {
    const file = new File(["content"], "test.txt", { type: "text/plain" });
    expect(validateFileType(file, ["text/plain"])).toBe(true);
    expect(validateFileType(file, ["image/png"])).toBe(false);
  });

  it("should validate wildcard MIME type", () => {
    const file = new File(["content"], "test.png", { type: "image/png" });
    expect(validateFileType(file, ["image/*"])).toBe(true);
    expect(validateFileType(file, ["text/*"])).toBe(false);
  });

  it("should validate file extension", () => {
    const file = new File(["content"], "test.txt", { type: "text/plain" });
    expect(validateFileType(file, [".txt"])).toBe(true);
    expect(validateFileType(file, [".pdf"])).toBe(false);
  });
});

describe("validateFileSize", () => {
  it("should accept any size when no max size provided", () => {
    const file = new File(["content"], "test.txt", { type: "text/plain" });
    expect(validateFileSize(file, undefined)).toBe(true);
  });

  it("should validate file size", () => {
    const file = new File(["x".repeat(1024)], "test.txt", { type: "text/plain" });
    expect(validateFileSize(file, 2048)).toBe(true);
    expect(validateFileSize(file, 512)).toBe(false);
  });
});

describe("validateFile", () => {
  it("should return null for valid file", () => {
    const file = new File(["content"], "test.txt", { type: "text/plain" });
    const result = validateFile(file, { accept: ["text/plain"], maxSize: 1024 });
    expect(result).toBeNull();
  });

  it("should return error for invalid type", () => {
    const file = new File(["content"], "test.txt", { type: "text/plain" });
    const result = validateFile(file, { accept: ["image/*"] });
    expect(result).toContain("File type not allowed");
  });

  it("should return error for too large file", () => {
    const file = new File(["x".repeat(2048)], "test.txt", { type: "text/plain" });
    const result = validateFile(file, { maxSize: 1024 });
    expect(result).toContain("File too large");
  });

  it("should use custom validator", () => {
    const file = new File(["content"], "test.txt", { type: "text/plain" });
    const validator = () => "Custom error";
    const result = validateFile(file, {}, validator);
    expect(result).toBe("Custom error");
  });
});

describe("processFiles", () => {
  it("should process valid files", () => {
    const files = [new File(["content"], "test.txt", { type: "text/plain" })];
    const result = processFiles(files, { accept: ["text/*"] });
    expect(result.valid).toHaveLength(1);
    expect(result.rejected).toHaveLength(0);
  });

  it("should reject invalid files", () => {
    const files = [new File(["content"], "test.txt", { type: "text/plain" })];
    const result = processFiles(files, { accept: ["image/*"] });
    expect(result.valid).toHaveLength(0);
    expect(result.rejected).toHaveLength(1);
  });

  it("should respect maxFiles limit", () => {
    const files = [
      new File(["1"], "1.txt", { type: "text/plain" }),
      new File(["2"], "2.txt", { type: "text/plain" }),
      new File(["3"], "3.txt", { type: "text/plain" }),
    ];
    const result = processFiles(files, { maxFiles: 2 });
    expect(result.valid).toHaveLength(2);
  });
});

describe("arrayMove", () => {
  it("should move item forward", () => {
    const array = [1, 2, 3, 4, 5];
    const result = arrayMove(array, 1, 3);
    expect(result).toEqual([1, 3, 4, 2, 5]);
  });

  it("should move item backward", () => {
    const array = [1, 2, 3, 4, 5];
    const result = arrayMove(array, 3, 1);
    expect(result).toEqual([1, 4, 2, 3, 5]);
  });

  it("should not mutate original array", () => {
    const array = [1, 2, 3];
    const result = arrayMove(array, 0, 2);
    expect(array).toEqual([1, 2, 3]);
    expect(result).not.toBe(array);
  });
});

describe("arrayInsert", () => {
  it("should insert item at index", () => {
    const array = [1, 2, 4];
    const result = arrayInsert(array, 2, 3);
    expect(result).toEqual([1, 2, 3, 4]);
  });

  it("should not mutate original array", () => {
    const array = [1, 2, 3];
    const result = arrayInsert(array, 1, 99);
    expect(array).toEqual([1, 2, 3]);
    expect(result).not.toBe(array);
  });
});

describe("arrayRemove", () => {
  it("should remove item at index", () => {
    const array = [1, 2, 3, 4];
    const result = arrayRemove(array, 2);
    expect(result).toEqual([1, 2, 4]);
  });

  it("should not mutate original array", () => {
    const array = [1, 2, 3];
    const result = arrayRemove(array, 1);
    expect(array).toEqual([1, 2, 3]);
    expect(result).not.toBe(array);
  });
});

describe("formatFileSize", () => {
  it("should format bytes", () => {
    expect(formatFileSize(0)).toBe("0 Bytes");
    expect(formatFileSize(500)).toBe("500 Bytes");
  });

  it("should format kilobytes", () => {
    expect(formatFileSize(1024)).toBe("1 KB");
    expect(formatFileSize(1536)).toBe("1.5 KB");
  });

  it("should format megabytes", () => {
    expect(formatFileSize(1048576)).toBe("1 MB");
    expect(formatFileSize(5242880)).toBe("5 MB");
  });

  it("should format gigabytes", () => {
    expect(formatFileSize(1073741824)).toBe("1 GB");
  });
});

describe("getFileExtension", () => {
  it("should extract extension", () => {
    expect(getFileExtension("file.txt")).toBe("txt");
    expect(getFileExtension("document.pdf")).toBe("pdf");
    expect(getFileExtension("archive.tar.gz")).toBe("gz");
  });

  it("should return empty string for no extension", () => {
    expect(getFileExtension("file")).toBe("");
  });

  it("should convert to lowercase", () => {
    expect(getFileExtension("FILE.TXT")).toBe("txt");
  });
});

describe("isImageFile", () => {
  it("should identify image files", () => {
    const image = new File(["content"], "test.png", { type: "image/png" });
    expect(isImageFile(image)).toBe(true);
  });

  it("should reject non-image files", () => {
    const text = new File(["content"], "test.txt", { type: "text/plain" });
    expect(isImageFile(text)).toBe(false);
  });
});
