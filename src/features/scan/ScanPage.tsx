// SPDX-License-Identifier: AGPL-3.0-or-later
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
        <select id="game-select" value={game} onChange={(e) => setGame(e.target.value as Game)}>
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
              No matches. Make sure you&apos;ve built the scan index from the Import / Export page.
            </p>
          ) : (
            <ol className="scan-matches">
              {result.matches.map((m) => (
                <li key={`${m.game}-${m.card_id}`} className="scan-match">
                  {m.image_url && (
                    <img src={m.image_url} alt={m.name} className="scan-match__thumb" />
                  )}
                  <div>
                    <strong>{m.name}</strong>
                    <span className="muted"> — {m.set_name}</span>
                    <br />
                    <span className="muted">
                      {Math.round(m.confidence * 100)}% · {m.card_id}
                    </span>
                  </div>
                </li>
              ))}
            </ol>
          )}
        </div>
      )}
    </section>
  );
}
