/**
 * App Store Tests
 * Tests Zustand state management
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "../../src/store/appStore";

describe("App Store", () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useAppStore.setState({
      messages: [],
      thoughts: [],
      uiSpec: null,
      partialUISpec: null,
      isLoading: false,
      isStreaming: false,
      error: null,
      generationThoughts: [],
      generationPreview: "",
      buildProgress: 0,
      appId: null,
    });
  });

  describe("Messages", () => {
    it("adds a message to the store", () => {
      const { addMessage } = useAppStore.getState();

      addMessage({
        type: "user",
        content: "Test message",
        timestamp: Date.now(),
      });

      const { messages } = useAppStore.getState();
      expect(messages).toHaveLength(1);
      expect(messages[0].content).toBe("Test message");
      expect(messages[0].type).toBe("user");
    });

    it("appends content to the last message", () => {
      const { addMessage, appendToLastMessage } = useAppStore.getState();

      addMessage({
        type: "assistant",
        content: "Hello",
        timestamp: Date.now(),
      });

      appendToLastMessage(" World");

      const { messages } = useAppStore.getState();
      expect(messages[0].content).toBe("Hello World");
    });

    it("creates a new message when appending to empty messages", () => {
      const { appendToLastMessage } = useAppStore.getState();

      appendToLastMessage("New message");

      const { messages } = useAppStore.getState();
      expect(messages).toHaveLength(1);
      expect(messages[0].content).toBe("New message");
      expect(messages[0].type).toBe("assistant");
    });

    it("maintains multiple messages in order", () => {
      const { addMessage } = useAppStore.getState();

      addMessage({ type: "user", content: "First", timestamp: 1 });
      addMessage({ type: "assistant", content: "Second", timestamp: 2 });
      addMessage({ type: "user", content: "Third", timestamp: 3 });

      const { messages } = useAppStore.getState();
      expect(messages).toHaveLength(3);
      expect(messages[0].content).toBe("First");
      expect(messages[1].content).toBe("Second");
      expect(messages[2].content).toBe("Third");
    });
  });

  describe("Thoughts", () => {
    it("adds a thought to the store", () => {
      const { addThought } = useAppStore.getState();

      addThought({
        content: "Thinking...",
        timestamp: Date.now(),
      });

      const { thoughts } = useAppStore.getState();
      expect(thoughts).toHaveLength(1);
      expect(thoughts[0].content).toBe("Thinking...");
    });

    it("maintains multiple thoughts", () => {
      const { addThought } = useAppStore.getState();

      addThought({ content: "Step 1", timestamp: 1 });
      addThought({ content: "Step 2", timestamp: 2 });

      const { thoughts } = useAppStore.getState();
      expect(thoughts).toHaveLength(2);
    });
  });

  describe("UI Spec", () => {
    it("sets UI spec and app ID", () => {
      const { setUISpec } = useAppStore.getState();
      const mockSpec = {
        type: "app",
        title: "Test App",
        layout: "vertical",
        components: [],
      };

      setUISpec(mockSpec, "app-123");

      const { uiSpec, appId } = useAppStore.getState();
      expect(uiSpec).toEqual(mockSpec);
      expect(appId).toBe("app-123");
    });

    it("clears UI spec", () => {
      const { setUISpec, clearUISpec } = useAppStore.getState();

      setUISpec(
        { type: "app", title: "Test", layout: "vertical", components: [] },
        "app-123"
      );
      clearUISpec();

      const { uiSpec, appId } = useAppStore.getState();
      expect(uiSpec).toBeNull();
      expect(appId).toBeNull();
    });

    it("sets partial UI spec during streaming", () => {
      const { setPartialUISpec } = useAppStore.getState();

      setPartialUISpec({ title: "Building App" });

      const { partialUISpec } = useAppStore.getState();
      expect(partialUISpec).toEqual({ title: "Building App" });
    });

    it("adds component to partial UI spec", () => {
      const { setPartialUISpec, addComponentToPartial } = useAppStore.getState();

      setPartialUISpec({ title: "App", components: [] });

      addComponentToPartial({
        type: "button",
        id: "btn-1",
        props: { text: "Click me" },
      });

      const { partialUISpec } = useAppStore.getState();
      expect(partialUISpec?.components).toHaveLength(1);
      expect(partialUISpec?.components?.[0].type).toBe("button");
    });

    it("initializes components array when adding to undefined partial spec", () => {
      const { addComponentToPartial } = useAppStore.getState();

      addComponentToPartial({
        type: "text",
        id: "txt-1",
        props: { content: "Hello" },
      });

      const { partialUISpec } = useAppStore.getState();
      expect(partialUISpec?.components).toHaveLength(1);
    });
  });

  describe("Loading States", () => {
    it("sets loading state", () => {
      const { setLoading } = useAppStore.getState();

      setLoading(true);
      expect(useAppStore.getState().isLoading).toBe(true);

      setLoading(false);
      expect(useAppStore.getState().isLoading).toBe(false);
    });

    it("sets streaming state", () => {
      const { setStreaming } = useAppStore.getState();

      setStreaming(true);
      expect(useAppStore.getState().isStreaming).toBe(true);

      setStreaming(false);
      expect(useAppStore.getState().isStreaming).toBe(false);
    });

    it("sets build progress", () => {
      const { setBuildProgress } = useAppStore.getState();

      setBuildProgress(50);
      expect(useAppStore.getState().buildProgress).toBe(50);

      setBuildProgress(100);
      expect(useAppStore.getState().buildProgress).toBe(100);
    });
  });

  describe("Errors", () => {
    it("sets error message", () => {
      const { setError } = useAppStore.getState();

      setError("Something went wrong");
      expect(useAppStore.getState().error).toBe("Something went wrong");
    });

    it("clears error by setting to null", () => {
      const { setError } = useAppStore.getState();

      setError("Error");
      setError(null);

      expect(useAppStore.getState().error).toBeNull();
    });
  });

  describe("Generation Preview", () => {
    it("appends to generation preview", () => {
      const { appendGenerationPreview } = useAppStore.getState();

      appendGenerationPreview("Hello");
      appendGenerationPreview(" World");

      expect(useAppStore.getState().generationPreview).toBe("Hello World");
    });

    it("clears generation preview", () => {
      const { appendGenerationPreview, clearGenerationPreview } = useAppStore.getState();

      appendGenerationPreview("Some content");
      clearGenerationPreview();

      expect(useAppStore.getState().generationPreview).toBe("");
    });
  });

  describe("Generation Thoughts", () => {
    it("adds generation thoughts", () => {
      const { addGenerationThought } = useAppStore.getState();

      addGenerationThought("Step 1");
      addGenerationThought("Step 2");

      const { generationThoughts } = useAppStore.getState();
      expect(generationThoughts).toEqual(["Step 1", "Step 2"]);
    });

    it("clears generation thoughts", () => {
      const { addGenerationThought, clearGenerationThoughts } = useAppStore.getState();

      addGenerationThought("Step 1");
      clearGenerationThoughts();

      expect(useAppStore.getState().generationThoughts).toEqual([]);
    });
  });

  describe("Reset State", () => {
    it("resets all state to initial values", () => {
      const { addMessage, addThought, setUISpec, setError, setLoading, resetState } =
        useAppStore.getState();

      // Modify state
      addMessage({ type: "user", content: "Test", timestamp: 1 });
      addThought({ content: "Thought", timestamp: 1 });
      setUISpec({ type: "app", title: "App", layout: "vertical", components: [] }, "app-123");
      setError("Error");
      setLoading(true);

      // Reset
      resetState();

      const state = useAppStore.getState();
      expect(state.messages).toEqual([]);
      expect(state.thoughts).toEqual([]);
      expect(state.uiSpec).toBeNull();
      expect(state.appId).toBeNull();
      expect(state.error).toBeNull();
      expect(state.isLoading).toBe(false);
      expect(state.isStreaming).toBe(false);
      expect(state.buildProgress).toBe(0);
      expect(state.generationPreview).toBe("");
      expect(state.generationThoughts).toEqual([]);
    });
  });
});

