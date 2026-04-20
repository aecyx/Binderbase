// SPDX-License-Identifier: AGPL-3.0-or-later
import { useEffect, useState } from "react";
import type { FormEvent, ReactElement } from "react";
import { api } from "../../lib/tauri";
import type { AppInfo } from "../../types";
import { isBinderbaseError } from "../../types";

interface Props {
  appInfo: AppInfo | null;
}

export function SettingsPage({ appInfo }: Props): ReactElement {
  const [apiKey, setApiKey] = useState("");
  const [savedKey, setSavedKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    api.settings
      .getPtcgApiKey()
      .then((key) => {
        if (!cancelled) {
          setSavedKey(key);
          setApiKey(key ?? "");
          setLoading(false);
        }
      })
      .catch((e) => {
        if (!cancelled) {
          setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function handleSave(evt: FormEvent) {
    evt.preventDefault();
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      const trimmed = apiKey.trim();
      await api.settings.setPtcgApiKey(trimmed);
      setSavedKey(trimmed || null);
      setApiKey(trimmed);
      setSuccess(trimmed ? "API key saved." : "API key cleared.");
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setSaving(false);
    }
  }

  const keyChanged = apiKey !== (savedKey ?? "");

  return (
    <section aria-labelledby="settings-heading">
      <h1 id="settings-heading">Settings</h1>

      {/* ---- Data directory ---- */}
      <h2>Data directory</h2>
      <p className="muted">
        Your collection, catalog cache, and prices are stored in a local SQLite database.
      </p>
      <div className="form-row">
        <span>Location</span>
        <code>{appInfo?.db_path ?? "…"}</code>
      </div>

      <hr />

      {/* ---- Pokémon TCG API key ---- */}
      <h2>Pokémon TCG API key</h2>
      <p className="muted">
        An API key is optional but recommended for the Pokémon TCG API. It raises rate limits and
        improves import reliability. Get a free key at{" "}
        <a href="https://pokemontcg.io" target="_blank" rel="noopener noreferrer">
          pokemontcg.io
        </a>
        .
      </p>

      {appInfo?.keyring_degraded && (
        <p className="notice" role="status">
          ⚠ OS keyring unavailable — the API key will not persist across launches.
        </p>
      )}

      {loading ? (
        <p role="status">Loading…</p>
      ) : (
        <form onSubmit={handleSave}>
          <div className="form-row">
            <label htmlFor="ptcg-api-key">API key</label>
            <input
              id="ptcg-api-key"
              type="password"
              value={apiKey}
              onChange={(e) => {
                setApiKey(e.target.value);
                setSuccess(null);
              }}
              placeholder="paste your key here"
              autoComplete="off"
              style={{ flex: 1, minWidth: "16rem" }}
            />
          </div>
          <div className="form-row">
            <button type="submit" data-variant="primary" disabled={saving || !keyChanged}>
              {saving ? "Saving…" : "Save"}
            </button>
            {savedKey && (
              <button
                type="button"
                disabled={saving}
                onClick={() => {
                  setApiKey("");
                  setSuccess(null);
                }}
              >
                Clear
              </button>
            )}
          </div>
          {success && (
            <p role="status" className="muted">
              {success}
            </p>
          )}
          {error && (
            <p role="alert" className="error">
              {error}
            </p>
          )}
        </form>
      )}
    </section>
  );
}
