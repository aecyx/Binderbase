// SPDX-License-Identifier: AGPL-3.0-or-later
import { useCallback, useEffect, useReducer, useState } from "react";
import type { FormEvent, ReactElement } from "react";
import { api } from "../../lib/tauri";
import type { CardCondition, CollectionEntry, Game, NewEntry } from "../../types";
import { GAME_DISPLAY_NAME, isBinderbaseError } from "../../types";

const CONDITIONS: { value: CardCondition; label: string }[] = [
  { value: "near_mint", label: "Near Mint" },
  { value: "lightly_played", label: "Lightly Played" },
  { value: "moderately_played", label: "Moderately Played" },
  { value: "heavily_played", label: "Heavily Played" },
  { value: "damaged", label: "Damaged" },
];

export function CollectionPage(): ReactElement {
  const [filter, setFilter] = useState<Game | "all">("all");
  const [entries, setEntries] = useState<CollectionEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [removing, setRemoving] = useState<string | null>(null);

  // Increment to trigger a re-fetch from the effect.
  const [refreshToken, bumpRefresh] = useReducer((n: number) => n + 1, 0);
  const refresh = useCallback(() => {
    setLoading(true);
    setError(null);
    bumpRefresh();
  }, []);

  // --- Add-card form state ---
  const [addGame, setAddGame] = useState<Game>("mtg");
  const [addCardId, setAddCardId] = useState("");
  const [addCondition, setAddCondition] = useState<CardCondition>("near_mint");
  const [addFoil, setAddFoil] = useState(false);
  const [addQty, setAddQty] = useState(1);
  const [addNotes, setAddNotes] = useState("");
  const [addBusy, setAddBusy] = useState(false);

  // Derive a fetch key so the effect re-runs when filter or refreshToken changes.
  // Loading/error state is reset eagerly in the handlers that trigger the change.
  const fetchKey = `${filter}:${refreshToken}`;

  useEffect(() => {
    let cancelled = false;
    api.collection
      .list(filter === "all" ? undefined : filter)
      .then((rows) => {
        if (!cancelled) {
          setEntries(rows);
          setError(null);
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
    // eslint-disable-next-line react-hooks/exhaustive-deps -- fetchKey encodes both deps
  }, [fetchKey]);

  function resetForm() {
    setAddCardId("");
    setAddCondition("near_mint");
    setAddFoil(false);
    setAddQty(1);
    setAddNotes("");
  }

  async function handleAdd(evt: FormEvent) {
    evt.preventDefault();
    if (!addCardId.trim()) return;
    setAddBusy(true);
    setError(null);
    try {
      const entry: NewEntry = {
        game: addGame,
        card_id: addCardId.trim(),
        condition: addCondition,
        foil: addFoil,
        quantity: addQty,
        notes: addNotes.trim() || null,
      };
      await api.collection.add(entry);
      resetForm();
      setShowForm(false);
      refresh();
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setAddBusy(false);
    }
  }

  async function handleRemove(entryId: string) {
    setRemoving(entryId);
    setError(null);
    try {
      await api.collection.remove(entryId);
      refresh();
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setRemoving(null);
    }
  }

  return (
    <section aria-labelledby="coll-heading">
      <h1 id="coll-heading">Collection</h1>

      <div className="form-row">
        <label htmlFor="coll-filter">Game</label>
        <select
          id="coll-filter"
          value={filter}
          onChange={(e) => {
            setLoading(true);
            setError(null);
            setFilter(e.target.value as Game | "all");
          }}
        >
          <option value="all">All games</option>
          <option value="mtg">{GAME_DISPLAY_NAME.mtg}</option>
          <option value="pokemon">{GAME_DISPLAY_NAME.pokemon}</option>
        </select>

        <button type="button" data-variant="primary" onClick={() => setShowForm((s) => !s)}>
          {showForm ? "Cancel" : "+ Add card"}
        </button>
      </div>

      {/* ---- Add-card form ---- */}
      {showForm && (
        <form className="add-card-form" onSubmit={handleAdd}>
          <div className="form-row">
            <label htmlFor="add-game">Game</label>
            <select
              id="add-game"
              value={addGame}
              onChange={(e) => setAddGame(e.target.value as Game)}
            >
              <option value="mtg">{GAME_DISPLAY_NAME.mtg}</option>
              <option value="pokemon">{GAME_DISPLAY_NAME.pokemon}</option>
            </select>
          </div>

          <div className="form-row">
            <label htmlFor="add-card-id">Card ID</label>
            <input
              id="add-card-id"
              type="text"
              required
              placeholder="e.g. Scryfall UUID or PTCG id"
              value={addCardId}
              onChange={(e) => setAddCardId(e.target.value)}
            />
          </div>

          <div className="form-row">
            <label htmlFor="add-condition">Condition</label>
            <select
              id="add-condition"
              value={addCondition}
              onChange={(e) => setAddCondition(e.target.value as CardCondition)}
            >
              {CONDITIONS.map((c) => (
                <option key={c.value} value={c.value}>
                  {c.label}
                </option>
              ))}
            </select>
          </div>

          <div className="form-row">
            <label htmlFor="add-qty">Quantity</label>
            <input
              id="add-qty"
              type="number"
              min={1}
              max={9999}
              value={addQty}
              onChange={(e) => setAddQty(Math.max(1, Number(e.target.value)))}
              style={{ width: "5rem" }}
            />

            <label htmlFor="add-foil" className="checkbox-label">
              <input
                id="add-foil"
                type="checkbox"
                checked={addFoil}
                onChange={(e) => setAddFoil(e.target.checked)}
              />
              Foil
            </label>
          </div>

          <div className="form-row">
            <label htmlFor="add-notes">Notes</label>
            <input
              id="add-notes"
              type="text"
              placeholder="optional"
              value={addNotes}
              onChange={(e) => setAddNotes(e.target.value)}
            />
          </div>

          <div className="form-row">
            <button type="submit" data-variant="primary" disabled={addBusy || !addCardId.trim()}>
              {addBusy ? "Adding…" : "Add to collection"}
            </button>
          </div>
        </form>
      )}

      {loading && <p role="status">Loading…</p>}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}

      {!loading && !error && entries.length === 0 && (
        <p className="muted">
          Your collection is empty. Use the &quot;+ Add card&quot; button to get started.
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
              <th scope="col">Notes</th>
              <th scope="col"></th>
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
                <td className="muted">{e.notes ?? ""}</td>
                <td>
                  <button
                    className="btn-remove"
                    disabled={removing === e.entry_id}
                    onClick={() => handleRemove(e.entry_id)}
                    aria-label={`Remove entry ${e.card_id}`}
                  >
                    {removing === e.entry_id ? "…" : "✕"}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
