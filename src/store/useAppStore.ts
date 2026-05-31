import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Session, Message, SessionFilters, IndexStats } from "../types";

interface AppState {
  sessions: Session[];
  selectedSessionId: string | null;
  messages: Message[];
  filters: SessionFilters;
  loading: boolean;
  initialLoading: boolean;
  activeSessionIds: Set<string>;

  loadSessions: () => Promise<void>;
  selectSession: (id: string) => Promise<void>;
  setFilters: (filters: Partial<SessionFilters>) => void;
  search: (query: string) => Promise<void>;
  loadMoreMessages: () => Promise<void>;
  refreshActiveSessions: () => Promise<void>;
  reindex: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  sessions: [],
  selectedSessionId: null,
  messages: [],
  filters: {},
  loading: false,
  initialLoading: true,
  activeSessionIds: new Set<string>(),

  loadSessions: async () => {
    set({ loading: true });
    try {
      const sessions = await invoke<Session[]>("get_sessions", {
        filters: get().filters,
        offset: 0,
        limit: 200,
      });
      set({ sessions, loading: false, initialLoading: false });
    } catch (e) {
      console.error("Failed to load sessions:", e);
      set({ loading: false, initialLoading: false });
    }
  },

  selectSession: async (id: string) => {
    set({ selectedSessionId: id, messages: [], loading: true });
    try {
      const messages = await invoke<Message[]>("get_session_messages", {
        sessionId: id,
        offset: 0,
        limit: 500,
      });
      set({ messages, loading: false });
    } catch (e) {
      console.error("Failed to load messages:", e);
      set({ loading: false });
    }
  },

  setFilters: (filters: Partial<SessionFilters>) => {
    const newFilters = { ...get().filters, ...filters };
    if (!filters.query && filters.query !== undefined) {
      delete newFilters.query;
    }
    if (!filters.agent && filters.agent !== undefined) {
      delete newFilters.agent;
    }
    if (!filters.project_path && filters.project_path !== undefined) {
      delete newFilters.project_path;
    }
    set({ filters: newFilters });
    get().loadSessions();
  },

  search: async (query: string) => {
    if (!query.trim()) {
      get().setFilters({ query: undefined as unknown as string });
      return;
    }
    get().setFilters({ query });
  },

  loadMoreMessages: async () => {
    const { messages, selectedSessionId } = get();
    if (!selectedSessionId) return;
    try {
      const more = await invoke<Message[]>("get_session_messages", {
        sessionId: selectedSessionId,
        offset: messages.length,
        limit: 500,
      });
      if (more.length > 0) {
        set({ messages: [...messages, ...more] });
      }
    } catch (e) {
      console.error("Failed to load more messages:", e);
    }
  },

  refreshActiveSessions: async () => {
    try {
      const activeIds = await invoke<string[]>("get_active_sessions");
      set({ activeSessionIds: new Set(activeIds) });
    } catch (e) {
      console.error("Failed to refresh active sessions:", e);
    }
  },

  reindex: async () => {
    try {
      const stats = await invoke<IndexStats>("reindex_all");
      console.log("Reindex complete:", stats);
      await get().loadSessions();
    } catch (e) {
      console.error("Failed to reindex:", e);
    }
  },
}));
