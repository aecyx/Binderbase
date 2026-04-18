// Typed wrappers around `@tauri-apps/api/core` invoke().
//
// Rationale: sprinkling bare `invoke("command_name", ...)` calls around the
// UI invites two kinds of drift:
//   * arg names/casing diverge from the Rust side
//   * return types decay to `unknown`
// Keeping the wrappers centralized means Rust-side renames fail the build
// here first, and callers get real types.

import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  Card,
  CollectionEntry,
  Game,
  NewEntry,
  Price,
  ScanResult,
} from "../types";

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),

  fetchCard: (game: Game, id: string) => invoke<Card>("fetch_card", { game, id }),

  collection: {
    list: (game?: Game) =>
      invoke<CollectionEntry[]>("collection_list", { game: game ?? null }),
    add: (entry: NewEntry) => invoke<CollectionEntry>("collection_add", { entry }),
    remove: (entryId: string) =>
      invoke<void>("collection_remove", { entryId }),
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
