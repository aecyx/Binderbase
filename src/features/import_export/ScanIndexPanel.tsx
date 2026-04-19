// SPDX-License-Identifier: AGPL-3.0-or-later
import { useCallback, useEffect, useState } from "react";
import { api } from "../../lib/tauri";
import type { Game, IndexProgress, IndexStatus } from "../../types";
import { GAME_DISPLAY_NAME, GAMES } from "../../types";

export function ScanIndexPanel() {
  const [status, setStatus] = useState<IndexStatus | null>(null);
  const [liveProgress, setLiveProgress] = useState<IndexProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStatus(await api.scanning.indexStatus());
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    api.scanning
      .indexStatus()
      .then(setStatus)
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    const unlisten = api.scanning.onIndexProgress((p) => {
      setLiveProgress(p);
      if (["finished", "cancelled", "failed"].includes(p.stage)) {
        setLiveProgress(null);
        refresh();
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refresh]);

  const progress = liveProgress ?? status?.progress;
  const inProgress = status?.in_progress ?? false;

  const handleBuild = async (game: Game) => {
    setError(null);
    try {
      await api.scanning.buildIndex(game);
      await refresh();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleCancel = async () => {
    try {
      await api.scanning.cancelBuildIndex();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const coverage: Record<Game, { hashed: number; total: number }> = {
    mtg: { hashed: status?.mtg_hashed ?? 0, total: status?.mtg_total ?? 0 },
    pokemon: { hashed: status?.pokemon_hashed ?? 0, total: status?.pokemon_total ?? 0 },
  };

  return (
    <div className="import-panel">
      <h2>Scan Index</h2>
      <p className="muted">
        Build a perceptual-hash index so card scanning can identify cards from images.
      </p>

      {error && <p className="error">{error}</p>}

      {inProgress && progress && (
        <div className="import-panel__progress">
          <div className="import-panel__stage">
            {progress.game && <strong>{GAME_DISPLAY_NAME[progress.game]}</strong>}{" "}
            <span className="muted">{progress.stage}</span>
          </div>
          {progress.total > 0 && (
            <>
              <progress
                value={progress.processed}
                max={progress.total}
                className="import-panel__bar"
              />
              <span className="muted">
                {progress.processed.toLocaleString()} / {progress.total.toLocaleString()} cards
              </span>
            </>
          )}
          {progress.message && <p className="muted">{progress.message}</p>}
          <button type="button" onClick={handleCancel}>
            Cancel
          </button>
        </div>
      )}

      <div className="import-panel__history">
        {GAMES.map((g) => {
          const c = coverage[g];
          const pct = c.total > 0 ? Math.round((c.hashed / c.total) * 100) : 0;
          return (
            <div key={g} className="import-run">
              <strong>{GAME_DISPLAY_NAME[g]}</strong>
              <span className="muted">
                {c.hashed.toLocaleString()} / {c.total.toLocaleString()} indexed ({pct}%)
              </span>
              <button
                type="button"
                className="btn-primary"
                disabled={inProgress || c.total === 0}
                onClick={() => handleBuild(g)}
              >
                {c.hashed === 0 ? "Build Index" : "Rebuild Index"}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
