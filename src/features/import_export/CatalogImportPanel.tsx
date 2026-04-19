// SPDX-License-Identifier: AGPL-3.0-or-later
import { useCallback, useEffect, useState } from "react";
import { api } from "../../lib/tauri";
import type { Game, ImportProgress, ImportRunSummary, ImportStatus } from "../../types";
import { GAME_DISPLAY_NAME } from "../../types";

export function CatalogImportPanel() {
  const [status, setStatus] = useState<ImportStatus | null>(null);
  const [liveProgress, setLiveProgress] = useState<ImportProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setStatus(await api.catalog.importStatus());
    } catch (e) {
      setError(String(e));
    }
  }, []);

  // Use live event data when available, fall back to status snapshot.
  const progress = liveProgress ?? status?.progress;

  useEffect(() => {
    api.catalog
      .importStatus()
      .then(setStatus)
      .catch((e) => setError(String(e)));
  }, []);

  // Subscribe to real-time progress events.
  useEffect(() => {
    const unlisten = api.catalog.onImportProgress((p) => {
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

  const handleStart = async (game?: Game) => {
    setStarting(true);
    setError(null);
    try {
      await api.catalog.importStart(game);
      await refresh();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setStarting(false);
    }
  };

  const handleCancel = async () => {
    try {
      await api.catalog.importCancel();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const inProgress = status?.in_progress ?? false;

  return (
    <div className="import-panel">
      <h2>Catalog Import</h2>
      <p className="muted">Download the full card catalog (and prices) for the games you play.</p>

      <div className="import-panel__actions">
        <button
          type="button"
          onClick={() => handleStart("mtg")}
          disabled={inProgress || starting}
          className="btn-primary"
        >
          {starting ? "Starting\u2026" : `Import ${GAME_DISPLAY_NAME.mtg}`}
        </button>
        <button
          type="button"
          onClick={() => handleStart("pokemon")}
          disabled={inProgress || starting}
          className="btn-primary"
        >
          {starting ? "Starting\u2026" : `Import ${GAME_DISPLAY_NAME.pokemon}`}
        </button>
        <button type="button" onClick={() => handleStart()} disabled={inProgress || starting}>
          {starting ? "Starting\u2026" : "Import all games"}
        </button>
        {inProgress && (
          <button type="button" onClick={handleCancel}>
            Cancel
          </button>
        )}
      </div>

      {error && <p className="error">{error}</p>}

      {inProgress && progress && (
        <div className="import-panel__progress">
          <div className="import-panel__stage">
            {progress.game && <strong>{GAME_DISPLAY_NAME[progress.game]}</strong>}{" "}
            <span className="muted">{stageLabel(progress.stage)}</span>
          </div>
          {progress.total !== null && progress.total > 0 && (
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
        </div>
      )}

      <div className="import-panel__history">
        {status?.last_mtg && <RunSummary run={status.last_mtg} />}
        {status?.last_pokemon && <RunSummary run={status.last_pokemon} />}
      </div>
    </div>
  );
}

function RunSummary({ run }: { run: ImportRunSummary }) {
  return (
    <div className="import-run">
      <strong>{GAME_DISPLAY_NAME[run.game]}</strong>
      <span className={`import-run__badge import-run__badge--${run.status}`}>{run.status}</span>
      <span className="muted">
        {run.cards_imported.toLocaleString()} cards
        {run.finished_at && ` \u00b7 ${new Date(run.finished_at).toLocaleString()}`}
      </span>
      {run.error_message && <p className="error">{run.error_message}</p>}
    </div>
  );
}

function stageLabel(stage: string): string {
  switch (stage) {
    case "fetching_bulk_index":
      return "Fetching catalog index\u2026";
    case "downloading":
      return "Downloading\u2026";
    case "parsing":
      return "Parsing\u2026";
    case "importing":
      return "Importing cards\u2026";
    case "finished":
      return "Finished";
    case "cancelled":
      return "Cancelled";
    case "failed":
      return "Failed";
    default:
      return stage;
  }
}
