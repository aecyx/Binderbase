// SPDX-License-Identifier: AGPL-3.0-or-later
// Typed wrappers around `@tauri-apps/api/core` invoke().
//
// Rationale: sprinkling bare `invoke("command_name", ...)` calls around the
// UI invites two kinds of drift:
//   * arg names/casing diverge from the Rust side
//   * return types decay to `unknown`
// Keeping the wrappers centralized means Rust-side renames fail the build
// here first, and callers get real types.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppInfo,
  Card,
  CollectionEntry,
  Game,
  ImportProgress,
  ImportStatus,
  NewEntry,
  Price,
  ScanResult,
} from "../types";

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),

  fetchCard: (game: Game, id: string) => invoke<Card>("fetch_card", { game, id }),

  catalog: {
    /**
     * Read a card straight from the local catalog. Returns `null` if the
     * catalog hasn't heard of it — callers typically fall through to
     * `fetchCard` (which is itself local-first) in that case.
     */
    get: (game: Game, cardId: string) => invoke<Card | null>("catalog_get", { game, cardId }),
    /**
     * Case-insensitive substring search against card names. Empty / whitespace
     * queries resolve to `[]`. `limit` is clamped server-side (default 25,
     * max 200).
     */
    search: (query: string, opts?: { game?: Game; limit?: number }) =>
      invoke<Card[]>("catalog_search", {
        game: opts?.game ?? null,
        query,
        limit: opts?.limit ?? null,
      }),
    importStart: () => invoke<void>("catalog_import_start"),
    importCancel: () => invoke<void>("catalog_import_cancel"),
    importStatus: () => invoke<ImportStatus>("catalog_import_status"),
    onImportProgress: (handler: (progress: ImportProgress) => void) =>
      listen<ImportProgress>("catalog:import:progress", (event) => handler(event.payload)),
  },

  collection: {
    list: (game?: Game) => invoke<CollectionEntry[]>("collection_list", { game: game ?? null }),
    add: (entry: NewEntry) => invoke<CollectionEntry>("collection_add", { entry }),
    remove: (entryId: string) => invoke<void>("collection_remove", { entryId }),
  },

  pricing: {
    getCached: (game: Game, cardId: string) =>
      invoke<Price[]>("pricing_get_cached", { game, cardId }),
  },

  scanning: {
    identify: (image: Uint8Array, gameHint?: Game) =>
      invoke<ScanResult>("scan_identify", {
        image: Array.from(image),
        gameHint: gameHint ?? null,
      }),
  },
};
