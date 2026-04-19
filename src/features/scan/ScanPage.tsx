// SPDX-License-Identifier: AGPL-3.0-or-later
import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";
import { api } from "../../lib/tauri";
import type { CardCondition, Game, IndexStatus, ScanMatch, ScanResult } from "../../types";
import { GAME_DISPLAY_NAME, isBinderbaseError } from "../../types";

const CONDITIONS: { value: CardCondition; label: string }[] = [
  { value: "near_mint", label: "Near Mint" },
  { value: "lightly_played", label: "Lightly Played" },
  { value: "moderately_played", label: "Moderately Played" },
  { value: "heavily_played", label: "Heavily Played" },
  { value: "damaged", label: "Damaged" },
];

/** Inline quick-add form shown next to a scan match. */
function AddFromScan({ match }: { match: ScanMatch }): ReactElement {
  const [open, setOpen] = useState(false);
  const [condition, setCondition] = useState<CardCondition>("near_mint");
  const [foil, setFoil] = useState(false);
  const [qty, setQty] = useState(1);
  const [busy, setBusy] = useState(false);
  const [added, setAdded] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function handleAdd() {
    setBusy(true);
    setErr(null);
    try {
      await api.collection.add({
        game: match.game,
        card_id: match.card_id,
        condition,
        foil,
        quantity: qty,
      });
      setAdded(true);
      setOpen(false);
    } catch (e) {
      setErr(isBinderbaseError(e) ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  if (added) return <span className="muted">Added ✓</span>;

  if (!open) {
    return (
      <button type="button" onClick={() => setOpen(true)}>
        + Collection
      </button>
    );
  }

  return (
    <span className="scan-match__add-form">
      <select value={condition} onChange={(e) => setCondition(e.target.value as CardCondition)}>
        {CONDITIONS.map((c) => (
          <option key={c.value} value={c.value}>
            {c.label}
          </option>
        ))}
      </select>
      <label>
        <input type="checkbox" checked={foil} onChange={(e) => setFoil(e.target.checked)} /> Foil
      </label>
      <input
        type="number"
        min={1}
        max={9999}
        value={qty}
        onChange={(e) => setQty(Math.max(1, Number(e.target.value)))}
        style={{ width: "3.5rem" }}
      />
      <button type="button" disabled={busy} onClick={handleAdd}>
        {busy ? "…" : "Add"}
      </button>
      <button type="button" onClick={() => setOpen(false)}>
        ✕
      </button>
      {err && <span className="error">{err}</span>}
    </span>
  );
}

export function ScanPage(): ReactElement {
  const [game, setGame] = useState<Game>("mtg");
  const [result, setResult] = useState<ScanResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [indexStatus, setIndexStatus] = useState<IndexStatus | null>(null);

  // Check scan index status on mount so we can warn if it's empty.
  useEffect(() => {
    api.scanning
      .indexStatus()
      .then(setIndexStatus)
      .catch(() => {
        // Non-fatal.
      });
  }, []);

  // -- webcam --
  const [camActive, setCamActive] = useState(false);
  const [camError, setCamError] = useState<string | null>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const stopCam = useCallback(() => {
    streamRef.current?.getTracks().forEach((t) => t.stop());
    streamRef.current = null;
    setCamActive(false);
  }, []);

  // Clean up on unmount.
  useEffect(() => stopCam, [stopCam]);

  async function startCam() {
    setCamError(null);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: "environment", width: { ideal: 1280 }, height: { ideal: 720 } },
        audio: false,
      });
      streamRef.current = stream;
      if (videoRef.current) {
        videoRef.current.srcObject = stream;
      }
      setCamActive(true);
    } catch (e) {
      setCamError(e instanceof Error ? e.message : String(e));
    }
  }

  /** Capture a single frame from the webcam and send it for identification. */
  async function captureFrame() {
    const video = videoRef.current;
    const canvas = canvasRef.current;
    if (!video || !canvas) return;
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(video, 0, 0);
    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/jpeg"));
    if (!blob) return;
    const buf = new Uint8Array(await blob.arrayBuffer());
    await identifyBytes(buf);
  }

  async function identifyBytes(buf: Uint8Array) {
    setError(null);
    setResult(null);
    setLoading(true);
    try {
      const res = await api.scanning.identify(buf, game);
      setResult(res);
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleFile(file: File) {
    const buf = new Uint8Array(await file.arrayBuffer());
    await identifyBytes(buf);
  }

  return (
    <section aria-labelledby="scan-heading">
      <h1 id="scan-heading">Scan a card</h1>

      {indexStatus && indexStatus.mtg_hashed === 0 && indexStatus.pokemon_hashed === 0 && (
        <div className="notice" role="status">
          <strong>Scan index not built yet.</strong> Go to{" "}
          <strong>Import / Export → Scan Index</strong> and build the index for at least one game
          before scanning.
        </div>
      )}

      <p className="muted">Use your webcam or load an image from your drive.</p>

      <div className="form-row">
        <label htmlFor="game-select">Game</label>
        <select id="game-select" value={game} onChange={(e) => setGame(e.target.value as Game)}>
          <option value="mtg">{GAME_DISPLAY_NAME.mtg}</option>
          <option value="pokemon">{GAME_DISPLAY_NAME.pokemon}</option>
        </select>
      </div>

      {/* -- Webcam section -- */}
      <div className="scan-source">
        {!camActive ? (
          <button type="button" data-variant="primary" onClick={startCam} disabled={loading}>
            Open webcam
          </button>
        ) : (
          <div className="webcam-container">
            {/* Webcam feed — no caption track needed for live video */}
            <video ref={videoRef} autoPlay playsInline className="webcam-preview" />
            <canvas ref={canvasRef} hidden />
            <div className="form-row">
              <button
                type="button"
                data-variant="primary"
                onClick={captureFrame}
                disabled={loading}
              >
                {loading ? "Identifying…" : "Capture & identify"}
              </button>
              <button type="button" onClick={stopCam}>
                Close webcam
              </button>
            </div>
          </div>
        )}
        {camError && (
          <p role="alert" className="error">
            Webcam error: {camError}
          </p>
        )}
      </div>

      <hr />

      {/* -- File upload fallback -- */}
      <div className="form-row">
        <label htmlFor="scan-file">Or upload an image</label>
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
                    <br />
                    <AddFromScan match={m} />
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
