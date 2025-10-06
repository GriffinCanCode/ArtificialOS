/**
 * Notes Tool Executor
 * Handles note creation, saving, loading, and list management
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { generatePrefixedId } from "../../../../../core/utils/id";
import { ExecutorContext, AsyncExecutor } from "../core/types";

interface Note {
  id: string;
  title: string;
  content: string;
  createdAt: number;
  updatedAt: number;
}

export class NotesExecutor implements AsyncExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  async execute(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case "create":
        return await this.createNote();

      case "save":
        return await this.saveNote(params);

      case "load":
        return await this.loadNote(params.noteId || params.id);

      case "delete":
        return await this.deleteNote(params.noteId || params.id);

      case "list":
        return await this.listNotes();

      default:
        logger.warn("Unknown notes action", { component: "NotesExecutor", action });
        return null;
    }
  }

  /**
   * Create a new note with generated ID
   */
  private async createNote(): Promise<Note> {
    const noteId = generatePrefixedId("note");
    const now = Date.now();

    const newNote: Note = {
      id: noteId,
      title: "",
      content: "",
      createdAt: now,
      updatedAt: now,
    };

    // Set current note state
    this.context.componentState.batch(() => {
      this.context.componentState.set("current-note-id", noteId);
      this.context.componentState.set("note-title", "");
      this.context.componentState.set("note-content", "");
    });

    logger.info("New note created", { component: "NotesExecutor", noteId });
    return newNote;
  }

  /**
   * Save current note to storage
   */
  private async saveNote(params: Record<string, any>): Promise<boolean> {
    try {
      // Get current note data from component state
      const noteId =
        params.noteId || this.context.componentState.get<string>("current-note-id");
      const title = this.context.componentState.get<string>("note-title", "");
      const content = this.context.componentState.get<string>("note-content", "");

      if (!noteId) {
        logger.warn("No note ID to save", { component: "NotesExecutor" });
        return false;
      }

      const now = Date.now();
      const existingNote = await this.loadNoteFromStorage(noteId);

      const note: Note = {
        id: noteId,
        title: title || "Untitled Note",
        content: content,
        createdAt: existingNote?.createdAt || now,
        updatedAt: now,
      };

      // Save to storage via service executor
      const response = await fetch("http://localhost:8000/services/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tool_id: "storage.set",
          params: {
            key: `note_${noteId}`,
            value: note,
          },
          app_id: this.context.appId,
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to save note: ${response.statusText}`);
      }

      const result = await response.json();

      if (!result.success) {
        throw new Error(result.error || "Storage save failed");
      }

      // Update notes list
      await this.updateNotesList();

      logger.info("Note saved successfully", { component: "NotesExecutor", noteId });
      return true;
    } catch (error) {
      logger.error("Failed to save note", error as Error, { component: "NotesExecutor" });
      return false;
    }
  }

  /**
   * Load a note by ID
   */
  private async loadNote(noteId: string): Promise<Note | null> {
    if (!noteId) return null;

    const note = await this.loadNoteFromStorage(noteId);
    if (!note) return null;

    // Set component state
    this.context.componentState.batch(() => {
      this.context.componentState.set("current-note-id", note.id);
      this.context.componentState.set("note-title", note.title);
      this.context.componentState.set("note-content", note.content);
      this.context.componentState.set("last-edited", this.formatDate(note.updatedAt));
    });

    return note;
  }

  /**
   * Delete a note
   */
  private async deleteNote(noteId: string): Promise<boolean> {
    if (!noteId) return false;

    try {
      const response = await fetch("http://localhost:8000/services/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tool_id: "storage.remove",
          params: {
            key: `note_${noteId}`,
          },
          app_id: this.context.appId,
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to delete note: ${response.statusText}`);
      }

      const result = await response.json();

      if (!result.success) {
        throw new Error(result.error || "Storage delete failed");
      }

      // Clear current note if it was deleted
      const currentId = this.context.componentState.get<string>("current-note-id");
      if (currentId === noteId) {
        this.context.componentState.batch(() => {
          this.context.componentState.set("current-note-id", null);
          this.context.componentState.set("note-title", "");
          this.context.componentState.set("note-content", "");
        });
      }

      // Update notes list
      await this.updateNotesList();

      logger.info("Note deleted", { component: "NotesExecutor", noteId });
      return true;
    } catch (error) {
      logger.error("Failed to delete note", error as Error, { component: "NotesExecutor" });
      return false;
    }
  }

  /**
   * List all notes
   */
  private async listNotes(): Promise<Note[]> {
    try {
      // Get all keys from storage
      const response = await fetch("http://localhost:8000/services/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tool_id: "storage.list",
          params: {},
          app_id: this.context.appId,
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to list notes: ${response.statusText}`);
      }

      const result = await response.json();

      if (!result.success) {
        throw new Error(result.error || "Storage list failed");
      }

      const keys = (result.data?.keys || []) as string[];
      const noteKeys = keys.filter((key) => key.startsWith("note_"));

      // Load all notes
      const notes: Note[] = [];
      for (const key of noteKeys) {
        const noteId = key.replace("note_", "");
        const note = await this.loadNoteFromStorage(noteId);
        if (note) {
          notes.push(note);
        }
      }

      // Sort by updated time (most recent first)
      notes.sort((a, b) => b.updatedAt - a.updatedAt);

      // Update notes list UI
      await this.updateNotesList(notes);

      return notes;
    } catch (error) {
      logger.error("Failed to list notes", error as Error, { component: "NotesExecutor" });
      return [];
    }
  }

  /**
   * Load a note from storage (helper)
   */
  private async loadNoteFromStorage(noteId: string): Promise<Note | null> {
    try {
      const response = await fetch("http://localhost:8000/services/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tool_id: "storage.get",
          params: {
            key: `note_${noteId}`,
          },
          app_id: this.context.appId,
        }),
      });

      if (!response.ok) {
        return null;
      }

      const result = await response.json();

      if (!result.success || !result.data?.value) {
        return null;
      }

      return result.data.value as Note;
    } catch (error) {
      logger.error("Failed to load note from storage", error as Error, {
        component: "NotesExecutor",
        noteId,
      });
      return null;
    }
  }

  /**
   * Update the notes list UI
   */
  private async updateNotesList(notes?: Note[]): Promise<void> {
    if (!notes) {
      notes = await this.listNotes();
      return; // listNotes calls this function with notes
    }

    // Update count
    this.context.componentState.set("notes-count", `${notes.length} NOTES`);

    // Create list items for UI
    const listItems = notes.map((note) => ({
      type: "container",
      id: `note-item-${note.id}`,
      props: {
        layout: "vertical",
        spacing: "none",
        padding: "small",
        style: {
          cursor: "pointer",
          padding: "0.75rem",
          borderBottom: "1px solid rgba(255,255,255,0.06)",
          transition: "all 0.2s ease",
        },
      },
      children: [
        {
          type: "text",
          id: `note-title-${note.id}`,
          props: {
            content: note.title || "Untitled Note",
            variant: "body",
            style: {
              fontWeight: "600",
              fontSize: "13px",
              color: "rgba(255,255,255,0.95)",
              marginBottom: "0.25rem",
            },
          },
        },
        {
          type: "text",
          id: `note-preview-${note.id}`,
          props: {
            content: this.getPreview(note.content),
            variant: "caption",
            color: "muted",
            style: {
              fontSize: "11px",
              lineHeight: "1.4",
            },
          },
        },
        {
          type: "text",
          id: `note-date-${note.id}`,
          props: {
            content: this.formatDate(note.updatedAt),
            variant: "caption",
            color: "muted",
            style: {
              fontSize: "10px",
              marginTop: "0.25rem",
            },
          },
        },
      ],
      on_event: {
        click: "notes.load",
      },
    }));

    // Update dynamic list via postMessage
    requestAnimationFrame(() => {
      window.postMessage(
        {
          type: "update_dynamic_lists",
          lists: {
            "notes-list": listItems,
          },
        },
        "*"
      );
    });
  }

  /**
   * Get preview text (first 80 chars)
   */
  private getPreview(content: string): string {
    if (!content) return "No content";
    const preview = content.substring(0, 80).trim();
    return preview.length < content.length ? `${preview}...` : preview;
  }

  /**
   * Format date for display
   */
  private formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;

    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 7) return `${diffDays}d ago`;

    // Format as date
    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: date.getFullYear() !== now.getFullYear() ? "numeric" : undefined,
    });
  }
}
