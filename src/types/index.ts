// SPDX-License-Identifier: AGPL-3.0-or-later
// Frontend-side mirrors of the Rust types exposed over Tauri commands.
//
// These are kept hand-written (not generated) for two reasons:
//  1. We only need a tiny surface; codegen is overkill for <200 LOC of types.
//  2. The handwritten file doubles as a contract check during PR review.
// If this grows, swap in `ts-rs`, `specta`, or similar.

export type Game = "mtg" | "pokemon";

export const GAMES: Game[] = ["mtg", "pokemon"];

export const GAME_DISPLAY_NAME: Record<Game, string> = {
  mtg: "Magic: The Gathering",
  pokemon: "Pokémon TCG",
};

export type CardCondition =
  | "near_mint"
  | "lightly_played"
  | "moderately_played"
  | "heavily_played"
  | "damaged";

export interface Card {
  game: Game;
  id: string;
  name: string;
  set_code: string;
  set_name: string;
  collector_number: string;
  image_url: string | null;
  rarity: string | null;
}

export interface CollectionEntry {
  entry_id: string;
  game: Game;
  card_id: string;
  condition: CardCondition;
  foil: boolean;
  quantity: number;
  notes: string | null;
  acquired_at: string | null;
  acquired_price_cents: number | null;
}

export interface NewEntry {
  game: Game;
  card_id: string;
  condition: CardCondition;
  foil?: boolean;
  quantity: number;
  notes?: string | null;
  acquired_at?: string | null;
  acquired_price_cents?: number | null;
}

export interface Price {
  game: Game;
  card_id: string;
  currency: string;
  source: string;
  cents: number;
  foil: boolean;
  fetched_at: string;
}

export interface ScanMatch {
  game: Game;
  card_id: string;
  name: string;
  set_name: string;
  image_url: string | null;
  confidence: number;
}

export interface ScanResult {
  matches: ScanMatch[];
  width: number;
  height: number;
}

export interface GameDescriptor {
  game: Game;
  data_source: string;
  pricing_source: string | null;
}

export interface AppInfo {
  name: string;
  version: string;
  db_path: string;
  supported_games: GameDescriptor[];
  keyring_degraded: boolean;
}

/** Discriminated error shape returned by Tauri commands. */
export type BinderbaseErrorKind =
  | "storage"
  | "network"
  | "card_not_found"
  | "unsupported_game"
  | "invalid_input"
  | "image_decode"
  | "internal";

export interface BinderbaseError {
  kind: BinderbaseErrorKind;
  message: string;
}

export function isBinderbaseError(e: unknown): e is BinderbaseError {
  return (
    typeof e === "object" &&
    e !== null &&
    "kind" in e &&
    "message" in e &&
    typeof (e as Record<string, unknown>).kind === "string"
  );
}

// ---------- Bulk import progress ----------

export interface ImportProgress {
  game: Game | null;
  stage: string;
  processed: number;
  total: number | null;
  message: string | null;
}

export interface ImportRunSummary {
  game: Game;
  started_at: string;
  finished_at: string | null;
  status: string;
  cards_imported: number;
  error_message: string | null;
}

export interface ImportStatus {
  in_progress: boolean;
  progress: ImportProgress | null;
  last_mtg: ImportRunSummary | null;
  last_pokemon: ImportRunSummary | null;
}

// ---------- Scan index ----------

export interface IndexProgress {
  game: Game;
  processed: number;
  total: number;
  stage: string;
  message: string | null;
}

export interface IndexStatus {
  in_progress: boolean;
  progress: IndexProgress | null;
  mtg_hashed: number;
  mtg_total: number;
  pokemon_hashed: number;
  pokemon_total: number;
}

// ---------- Collection CSV import/export ----------

export interface CsvRowError {
  line: number;
  message: string;
}

export interface CsvPreviewEntry {
  game: Game;
  card_id: string;
  name: string;
  condition: string;
  foil: boolean;
  quantity: number;
}

export interface CsvImportPreview {
  total_rows: number;
  valid_rows: number;
  errors: CsvRowError[];
  sample: CsvPreviewEntry[];
}

export interface CsvImportResult {
  imported: number;
  skipped: number;
  errors: CsvRowError[];
}

// ---------- Price refresh ----------

export interface RefreshProgress {
  done: number;
  total: number;
  card_id: string;
  game: Game;
  ok: boolean;
  error: string | null;
}
