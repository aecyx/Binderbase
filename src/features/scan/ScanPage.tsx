import { useState } from "react";
import type { ReactElement } from "react";
import { api } from "../../lib/tauri";
import type { Game, ScanResult } from "../../types";
import { GAME_DISPLAY_NAME, isBinderbaseError } from "../../types";

export function ScanPage(): ReactElement {
  const [game, setGame] = useState<Game>("mtg");
  const [result, setResult] = useState<ScanResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleFile(file: File) {
    setError(null);
    setResult(null);
    setLoading(true);
    try {
      const buf = new Uint8Array(await file.arrayBuffer());
      const res = await api.scanning.identify(buf, game);
      setResult(res);
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section aria-labelledby="scan-heading">
      <h1 id="scan-heading">Scan a card</h1>
      <p className="muted">
        Load an image from your drive. Webcam and phone-camera capture come later.
      </p>

      <div className="form-row">
        <label htmlFor="game-select">Game</label>
        <select
          id="game-select"
          value={game}
          onChange={(e) => setGame(e.target.value as Game)}
        >
          <option value="mtg">{GAME_DISPLAY_NAME.mtg}</option>
          <option value="pokemon">{GAME_DISPLAY_NAME.pokemon}</option>
        </select>
      </div>

      <div className="form-row">
        <label htmlFor="scan-file">Card image</label>
        <input
          id="scan-file"
          type="file"
          accept="image/jpeg,image/png,image/webp"
          disabled={loading}
          onChange={(e) => {
            const f = e.currentTarget.files?.[0];
            if (f) handleFile(f);
          }}
        />
      </div>

      {loading && <p role="status">Identifying…</p>}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}

      {result && (
        <div className="result">
          <p>
            Decoded image: {result.width}×{result.height}.
          </p>
          {result.matches.length === 0 ? (
            <p className="muted">
              No matches yet — identification model is still a work in progress.
              See <code>src-tauri/src/scanning/mod.rs</code>.
            </p>
          ) : (
            <ol>
              {result.matches.map((m) => (
                <li key={m.card_id}>
                  <code>{m.card_id}</code> ({Math.round(m.confidence * 100)}%)
                </li>
              ))}
            </ol>
          )}
        </div>
      )}
    </section>
  );
}
