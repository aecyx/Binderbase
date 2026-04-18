// SPDX-License-Identifier: AGPL-3.0-or-later
import { useEffect, useState } from "react";
import type { ReactElement } from "react";
import { api } from "../../lib/tauri";
import type { CollectionEntry, Game } from "../../types";
import { GAME_DISPLAY_NAME, isBinderbaseError } from "../../types";

export function CollectionPage(): ReactElement {
  const [filter, setFilter] = useState<Game | "all">("all");
  const [entries, setEntries] = useState<CollectionEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  function changeFilter(value: Game | "all") {
    setLoading(true);
    setError(null);
    setFilter(value);
  }

  useEffect(() => {
    let cancelled = false;
    api.collection
      .list(filter === "all" ? undefined : filter)
      .then((rows) => {
        if (!cancelled) {
          setEntries(rows);
          setError(null);
        }
      })
      .catch((e) => {
        if (!cancelled) {
          setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [filter]);

  return (
    <section aria-labelledby="coll-heading">
      <h1 id="coll-heading">Collection</h1>

      <div className="form-row">
        <label htmlFor="coll-filter">Game</label>
        <select
          id="coll-filter"
          value={filter}
          onChange={(e) => changeFilter(e.target.value as Game | "all")}
        >
          <option value="all">All games</option>
          <option value="mtg">{GAME_DISPLAY_NAME.mtg}</option>
          <option value="pokemon">{GAME_DISPLAY_NAME.pokemon}</option>
        </select>
      </div>

      {loading && <p role="status">Loading…</p>}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}

      {!loading && !error && entries.length === 0 && (
        <p className="muted">
          Your collection is empty. Scan a card or import a CSV to get started.
        </p>
      )}

      {entries.length > 0 && (
        <table className="collection-table">
          <thead>
            <tr>
              <th scope="col">Game</th>
              <th scope="col">Card ID</th>
              <th scope="col">Condition</th>
              <th scope="col">Foil</th>
              <th scope="col">Qty</th>
            </tr>
          </thead>
          <tbody>
            {entries.map((e) => (
              <tr key={e.entry_id}>
                <td>{GAME_DISPLAY_NAME[e.game]}</td>
                <td>
                  <code>{e.card_id}</code>
                </td>
                <td>{e.condition.replace("_", " ")}</td>
                <td>{e.foil ? "yes" : ""}</td>
                <td>{e.quantity}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
