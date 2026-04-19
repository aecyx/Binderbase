// SPDX-License-Identifier: AGPL-3.0-or-later
import { useCallback, useEffect, useReducer, useState } from "react";
import type { FormEvent, ReactElement } from "react";
import { CardSearch } from "../../components/CardSearch";
import { api } from "../../lib/tauri";
import type {
  Card,
  CardCondition,
  CollectionEntry,
  Game,
  NewEntry,
  Price,
  RefreshProgress,
} from "../../types";
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

  // Price cache: key = "game:card_id", value = best non-foil price or first available
  const [priceMap, setPriceMap] = useState<Record<string, Price | null>>({});
  const [refreshingPrices, setRefreshingPrices] = useState(false);
  const [refreshProgress, setRefreshProgress] = useState<RefreshProgress | null>(null);

  // Increment to trigger a re-fetch from the effect.
  const [refreshToken, bumpRefresh] = useReducer((n: number) => n + 1, 0);
  const refresh = useCallback(() => {
    setLoading(true);
    setError(null);
    bumpRefresh();
  }, []);

  // --- Add-card form state ---
  const [addGame, setAddGame] = useState<Game>("mtg");
  const [addCard, setAddCard] = useState<Card | null>(null);
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

  // Load cached prices for all distinct cards whenever entries change.
  useEffect(() => {
    let cancelled = false;
    const seen = new Set<string>();
    const toFetch: { game: Game; card_id: string }[] = [];
    for (const e of entries) {
      const key = `${e.game}:${e.card_id}`;
      if (!seen.has(key)) {
        seen.add(key);
        toFetch.push({ game: e.game, card_id: e.card_id });
      }
    }
    if (toFetch.length === 0) return;
    Promise.all(
      toFetch.map(({ game, card_id }) =>
        api.pricing
          .getCached(game, card_id)
          .then((prices) => ({ key: `${game}:${card_id}`, prices }))
          .catch(() => ({ key: `${game}:${card_id}`, prices: [] as Price[] })),
      ),
    ).then((results) => {
      if (cancelled) return;
      const map: Record<string, Price | null> = {};
      for (const { key, prices } of results) {
        // Prefer non-foil USD, then any USD, then first available
        const best =
          prices.find((p) => p.currency === "usd" && !p.foil) ??
          prices.find((p) => p.currency === "usd") ??
          prices[0] ??
          null;
        map[key] = best;
      }
      setPriceMap(map);
    });
    return () => {
      cancelled = true;
    };
  }, [entries]);

  async function handleRefreshAll() {
    setRefreshingPrices(true);
    setRefreshProgress(null);
    setError(null);
    const unlisten = await api.pricing.onRefreshProgress((p) => {
      setRefreshProgress(p);
      if (p.done === p.total) {
        setRefreshingPrices(false);
        refresh(); // reload collection + prices
      }
    });
    try {
      await api.pricing.refreshCollection(filter === "all" ? undefined : filter);
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
      setRefreshingPrices(false);
    } finally {
      unlisten();
    }
  }

  function formatCents(cents: number, currency: string): string {
    return (cents / 100).toLocaleString(undefined, {
      style: "currency",
      currency,
    });
  }

  function resetForm() {
    setAddCard(null);
    setAddCondition("near_mint");
    setAddFoil(false);
    setAddQty(1);
    setAddNotes("");
  }

  async function handleAdd(evt: FormEvent) {
    evt.preventDefault();
    if (!addCard) return;
    setAddBusy(true);
    setError(null);
    try {
      const entry: NewEntry = {
        game: addGame,
        card_id: addCard.id,
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

        {entries.length > 0 && (
          <button type="button" disabled={refreshingPrices} onClick={handleRefreshAll}>
            {refreshingPrices
              ? `Refreshing${refreshProgress ? ` ${refreshProgress.done}/${refreshProgress.total}` : "…"}`
              : "Refresh all prices"}
          </button>
        )}
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
            <label htmlFor="add-card-search">Card</label>
            <CardSearch
              game={addGame}
              onSelect={setAddCard}
              placeholder="Search by card name\u2026"
              id="add-card-search"
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
            <button type="submit" data-variant="primary" disabled={addBusy || !addCard}>
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
              <th scope="col">Price</th>
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
                <td className="muted">
                  {(() => {
                    const p = priceMap[`${e.game}:${e.card_id}`];
                    return p ? formatCents(p.cents, p.currency) : "—";
                  })()}
                </td>
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
