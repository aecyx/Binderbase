// SPDX-License-Identifier: AGPL-3.0-or-later
import { useRef, useState } from "react";
import type { ReactElement } from "react";
import { api } from "../../lib/tauri";
import type { CsvImportPreview, CsvImportResult, Game } from "../../types";
import { GAME_DISPLAY_NAME, isBinderbaseError } from "../../types";

export function CollectionCsvPanel(): ReactElement {
  return (
    <div>
      <h2>Collection Import / Export</h2>
      <p className="muted">Export your collection as CSV or import entries from a CSV file.</p>
      <ExportSection />
      <ImportSection />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

function ExportSection(): ReactElement {
  const [game, setGame] = useState<Game | "">("");
  const [exporting, setExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [count, setCount] = useState<number | null>(null);

  const handleExport = async () => {
    setExporting(true);
    setError(null);
    setCount(null);
    try {
      const csv = await api.collection.exportCsv(game || undefined);
      // Count data rows (subtract header).
      const rows = csv.trim().split("\n").length - 1;
      setCount(rows);

      // Trigger a browser download.
      const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = game ? `binderbase-${game}-collection.csv` : "binderbase-collection.csv";
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="csv-section">
      <h3>Export</h3>
      <div className="form-row">
        <label htmlFor="export-game">Game filter</label>
        <select
          id="export-game"
          value={game}
          onChange={(e) => setGame(e.target.value as Game | "")}
        >
          <option value="">All games</option>
          <option value="mtg">{GAME_DISPLAY_NAME.mtg}</option>
          <option value="pokemon">{GAME_DISPLAY_NAME.pokemon}</option>
        </select>
      </div>
      <button type="button" onClick={handleExport} disabled={exporting} className="btn-primary">
        {exporting ? "Exporting\u2026" : "Export CSV"}
      </button>
      {count !== null && (
        <p className="muted">
          Exported {count.toLocaleString()} {count === 1 ? "entry" : "entries"}.
        </p>
      )}
      {error && <p className="error">{error}</p>}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

type ImportStage = "idle" | "previewing" | "previewed" | "applying" | "done";

function ImportSection(): ReactElement {
  const fileRef = useRef<HTMLInputElement>(null);
  const [csvText, setCsvText] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [stage, setStage] = useState<ImportStage>("idle");
  const [preview, setPreview] = useState<CsvImportPreview | null>(null);
  const [result, setResult] = useState<CsvImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reset = () => {
    setCsvText(null);
    setFileName(null);
    setStage("idle");
    setPreview(null);
    setResult(null);
    setError(null);
    if (fileRef.current) fileRef.current.value = "";
  };

  const handleFile = async (file: File) => {
    setError(null);
    setPreview(null);
    setResult(null);
    setStage("previewing");

    try {
      const text = await file.text();
      setCsvText(text);
      setFileName(file.name);
      const p = await api.collection.importPreview(text);
      setPreview(p);
      setStage("previewed");
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
      setStage("idle");
    }
  };

  const handleApply = async () => {
    if (!csvText) return;
    setStage("applying");
    setError(null);
    try {
      const r = await api.collection.importApply(csvText);
      setResult(r);
      setStage("done");
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
      setStage("previewed");
    }
  };

  return (
    <div className="csv-section">
      <h3>Import</h3>

      <div className="form-row">
        <label htmlFor="import-file">CSV file</label>
        <input
          ref={fileRef}
          id="import-file"
          type="file"
          accept=".csv,text/csv"
          disabled={stage === "previewing" || stage === "applying"}
          onChange={(e) => {
            const f = e.currentTarget.files?.[0];
            if (f) handleFile(f);
          }}
        />
      </div>

      {stage === "previewing" && <p role="status">Validating\u2026</p>}

      {preview && stage === "previewed" && (
        <div className="csv-preview">
          <p>
            <strong>{fileName}</strong> &mdash; {preview.valid_rows.toLocaleString()} of{" "}
            {preview.total_rows.toLocaleString()} rows valid.
          </p>

          {preview.errors.length > 0 && (
            <details>
              <summary className="error">
                {preview.errors.length} {preview.errors.length === 1 ? "error" : "errors"}
              </summary>
              <ul className="csv-errors">
                {preview.errors.map((e, i) => (
                  <li key={i}>
                    <strong>Line {e.line}:</strong> {e.message}
                  </li>
                ))}
              </ul>
            </details>
          )}

          {preview.sample.length > 0 && (
            <details open>
              <summary>Preview ({Math.min(preview.sample.length, 50)} rows)</summary>
              <table className="csv-preview-table">
                <thead>
                  <tr>
                    <th>Game</th>
                    <th>Card</th>
                    <th>Condition</th>
                    <th>Foil</th>
                    <th>Qty</th>
                  </tr>
                </thead>
                <tbody>
                  {preview.sample.map((row, i) => (
                    <tr key={i}>
                      <td>{GAME_DISPLAY_NAME[row.game]}</td>
                      <td>
                        {row.name || row.card_id}
                        {row.name && <span className="muted"> ({row.card_id})</span>}
                      </td>
                      <td>{row.condition}</td>
                      <td>{row.foil ? "Yes" : "No"}</td>
                      <td>{row.quantity}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </details>
          )}

          <div className="csv-section__actions">
            <button
              type="button"
              onClick={handleApply}
              disabled={preview.valid_rows === 0}
              className="btn-primary"
            >
              Import {preview.valid_rows.toLocaleString()}{" "}
              {preview.valid_rows === 1 ? "entry" : "entries"}
            </button>
            <button type="button" onClick={reset}>
              Cancel
            </button>
          </div>
        </div>
      )}

      {stage === "applying" && <p role="status">Importing\u2026</p>}

      {result && stage === "done" && (
        <div className="csv-result">
          <p>
            Imported <strong>{result.imported.toLocaleString()}</strong>{" "}
            {result.imported === 1 ? "entry" : "entries"}.
            {result.skipped > 0 && <span> Skipped {result.skipped.toLocaleString()}.</span>}
          </p>
          {result.errors.length > 0 && (
            <details>
              <summary className="error">
                {result.errors.length} {result.errors.length === 1 ? "error" : "errors"}
              </summary>
              <ul className="csv-errors">
                {result.errors.map((e, i) => (
                  <li key={i}>
                    <strong>Line {e.line}:</strong> {e.message}
                  </li>
                ))}
              </ul>
            </details>
          )}
          <button type="button" onClick={reset}>
            Import another file
          </button>
        </div>
      )}

      {error && <p className="error">{error}</p>}
    </div>
  );
}
