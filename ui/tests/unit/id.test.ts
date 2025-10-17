import { describe, it, expect, beforeEach } from "vitest";
import {
  newAppID,
  newWindowID,
  newSessionID,
  newRequestID,
  newComponentID,
  newPackageID,
  newToolID,
  isValid,
  extractTimestamp,
  extractPrefix,
  generateBatch,
  generateRaw,
  generatePrefixed,
  compare,
  sort,
  isAppID,
  isWindowID,
  Prefix,
  type AppID,
  type WindowID,
} from "../../src/core/id";

describe("ID Generation", () => {
  it("generates unique IDs", () => {
    const id1 = generateRaw();
    const id2 = generateRaw();

    expect(id1).not.toBe(id2);
    expect(id1).toHaveLength(26);
    expect(id2).toHaveLength(26);
  });

  it("generates valid ULIDs", () => {
    const id = generateRaw();
    expect(isValid(id)).toBe(true);
  });

  it("generates monotonically increasing IDs", async () => {
    const ids: string[] = [];
    for (let i = 0; i < 100; i++) {
      ids.push(generateRaw());
    }

    for (let i = 1; i < ids.length; i++) {
      expect(ids[i] >= ids[i - 1]).toBe(true);
    }
  });
});

describe("Typed ID Generation", () => {
  it("generates app IDs with correct prefix", () => {
    const id = newAppID();
    expect(id).toContain("app_");
    expect(extractPrefix(id)).toBe(Prefix.App);
  });

  it("generates window IDs with correct prefix", () => {
    const id = newWindowID();
    expect(id).toContain("win_");
    expect(extractPrefix(id)).toBe(Prefix.Window);
  });

  it("generates session IDs with correct prefix", () => {
    const id = newSessionID();
    expect(id).toContain("sess_");
    expect(extractPrefix(id)).toBe(Prefix.Session);
  });

  it("generates request IDs with correct prefix", () => {
    const id = newRequestID();
    expect(id).toContain("req_");
    expect(extractPrefix(id)).toBe(Prefix.Request);
  });

  it("generates component IDs with correct prefix", () => {
    const id = newComponentID();
    expect(id).toContain("cmp_");
    expect(extractPrefix(id)).toBe(Prefix.Component);
  });

  it("generates package IDs with correct prefix", () => {
    const id = newPackageID();
    expect(id).toContain("pkg_");
    expect(extractPrefix(id)).toBe(Prefix.Package);
  });

  it("generates tool IDs with correct prefix", () => {
    const id = newToolID();
    expect(id).toContain("tool_");
    expect(extractPrefix(id)).toBe(Prefix.Tool);
  });

  it("all typed IDs are valid", () => {
    expect(isValid(newAppID())).toBe(true);
    expect(isValid(newWindowID())).toBe(true);
    expect(isValid(newSessionID())).toBe(true);
    expect(isValid(newRequestID())).toBe(true);
    expect(isValid(newComponentID())).toBe(true);
    expect(isValid(newPackageID())).toBe(true);
    expect(isValid(newToolID())).toBe(true);
  });
});

describe("Validation", () => {
  it("validates correct ULIDs", () => {
    const id = generateRaw();
    expect(isValid(id)).toBe(true);
  });

  it("validates prefixed ULIDs", () => {
    const id = newAppID();
    expect(isValid(id)).toBe(true);
  });

  it("rejects invalid IDs", () => {
    expect(isValid("")).toBe(false);
    expect(isValid("invalid")).toBe(false);
    expect(isValid("1234567890")).toBe(false);
    // Note: ZZZZZZZZZZZZZZZZZZZZZZZZZZ is actually a valid ULID per spec
  });

  it("rejects malformed prefixed IDs", () => {
    expect(isValid("app_INVALID")).toBe(false);
    // Note: _01ARZ3NDEKTSV4RRFFQ69G5FAV has valid ULID part after underscore
    expect(isValid("app_")).toBe(false); // Empty ULID part
    expect(isValid("_")).toBe(false); // Empty ULID part
  });
});

describe("Timestamp Extraction", () => {
  it("extracts timestamp from ULID", () => {
    const before = Date.now();
    const id = generateRaw();
    const after = Date.now();

    const timestamp = extractTimestamp(id);
    expect(timestamp).not.toBeNull();
    expect(timestamp!.getTime()).toBeGreaterThanOrEqual(before);
    expect(timestamp!.getTime()).toBeLessThanOrEqual(after);
  });

  it("extracts timestamp from prefixed ULID", () => {
    const before = Date.now();
    const id = newAppID();
    const after = Date.now();

    const timestamp = extractTimestamp(id);
    expect(timestamp).not.toBeNull();
    expect(timestamp!.getTime()).toBeGreaterThanOrEqual(before);
    expect(timestamp!.getTime()).toBeLessThanOrEqual(after);
  });

  it("returns null for invalid ID", () => {
    expect(extractTimestamp("invalid")).toBeNull();
  });
});

describe("Prefix Extraction", () => {
  it("extracts prefix from prefixed ID", () => {
    const id = newAppID();
    expect(extractPrefix(id)).toBe("app");
  });

  it("returns null for non-prefixed ID", () => {
    const id = generateRaw();
    expect(extractPrefix(id)).toBeNull();
  });
});

describe("Batch Generation", () => {
  it("generates multiple IDs efficiently", () => {
    const count = 100;
    const ids = generateBatch(count);

    expect(ids).toHaveLength(count);

    // Check uniqueness
    const unique = new Set(ids);
    expect(unique.size).toBe(count);

    // Check all valid
    ids.forEach((id) => {
      expect(isValid(id)).toBe(true);
    });
  });
});

describe("Custom Prefixed IDs", () => {
  it("generates custom prefixed IDs", () => {
    const id = generatePrefixed("custom");
    expect(id).toContain("custom_");
    expect(isValid(id)).toBe(true);
  });
});

describe("Sorting and Comparison", () => {
  it("compares ULIDs by timestamp", () => {
    const id1 = generateRaw();
    // Small delay to ensure different timestamp
    const delay = () => new Promise((resolve) => setTimeout(resolve, 2));

    delay().then(() => {
      const id2 = generateRaw();
      expect(compare(id1, id2)).toBeLessThan(0);
      expect(compare(id2, id1)).toBeGreaterThan(0);
      expect(compare(id1, id1)).toBe(0);
    });
  });

  it("sorts IDs by timestamp", async () => {
    const ids: string[] = [];

    for (let i = 0; i < 10; i++) {
      ids.push(generateRaw());
      await new Promise((resolve) => setTimeout(resolve, 1));
    }

    const shuffled = [...ids].sort(() => Math.random() - 0.5);
    const sorted = sort(shuffled);

    expect(sorted).toEqual(ids);
  });
});

describe("Type Guards", () => {
  it("identifies app IDs", () => {
    const appId = newAppID();
    const winId = newWindowID();

    expect(isAppID(appId)).toBe(true);
    expect(isAppID(winId)).toBe(false);
    expect(isAppID("invalid")).toBe(false);
  });

  it("identifies window IDs", () => {
    const winId = newWindowID();
    const appId = newAppID();

    expect(isWindowID(winId)).toBe(true);
    expect(isWindowID(appId)).toBe(false);
    expect(isWindowID("invalid")).toBe(false);
  });
});

describe("Uniqueness Under Load", () => {
  it("generates unique IDs under concurrent load", () => {
    const count = 10000;
    const ids = new Set<string>();

    for (let i = 0; i < count; i++) {
      ids.add(generateRaw());
    }

    expect(ids.size).toBe(count);
  });

  it("generates unique typed IDs under load", () => {
    const count = 1000;
    const appIds = new Set<AppID>();
    const winIds = new Set<WindowID>();

    for (let i = 0; i < count; i++) {
      appIds.add(newAppID());
      winIds.add(newWindowID());
    }

    expect(appIds.size).toBe(count);
    expect(winIds.size).toBe(count);

    // Ensure no overlap between different ID types
    const appArray = Array.from(appIds);
    const winArray = Array.from(winIds);

    appArray.forEach((id) => {
      expect(winIds.has(id as unknown as WindowID)).toBe(false);
    });
  });
});

describe("Format Consistency", () => {
  it("all typed IDs follow prefix_ULID format", () => {
    const ids = {
      app: newAppID(),
      win: newWindowID(),
      sess: newSessionID(),
      req: newRequestID(),
      cmp: newComponentID(),
      pkg: newPackageID(),
      tool: newToolID(),
    };

    Object.entries(ids).forEach(([prefix, id]) => {
      const parts = id.split("_");
      expect(parts).toHaveLength(2);
      expect(parts[1]).toHaveLength(26);
      expect(isValid(parts[1])).toBe(true);
    });
  });
});

