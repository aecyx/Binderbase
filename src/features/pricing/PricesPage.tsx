// SPDX-License-Identifier: AGPL-3.0-or-later
import { useState } from "react";
import type { ReactElement } from "react";
import { CardSearch } from "../../components/CardSearch";
import { api } from "../../lib/tauri";
import type { Card, Game, Price } from "../../types";
import { GAME_DISPLAY_NAME, isBinderbaseError } from "../../types";

export function PricesPage(): ReactElement {
  const [game, setGame] = useState<Game>("mtg");
  const [card, setCard] = useState<Card | null>(null);
  const [prices, setPrices] = useState<Price[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  async function handleSelect(selected: Card) {
    setError(null);
    setCard(null);
    setPrices([]);
    setLoading(true);
    try {
      const [c, p] = await Promise.all([
        api.fetchCard(selected.game, selected.id),
        api.pricing.getCached(selected.game, selected.id),
      ]);
      setCard(c);
      setPrices(p);
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleRefresh() {
    if (!card) return;
    setRefreshing(true);
    setError(null);
    try {
      const fresh = await api.pricing.refresh(card.game, card.id);
      setPrices(fresh);
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setRefreshing(false);
    }
  }

  return (
    <section aria-labelledby="prices-heading">
      <h1 id="prices-heading">Prices</h1>
      <p className="muted">
        Live catalog lookup + cached price history. Prices are pulled from Scryfall (MTG) or the
        Pokémon TCG API.
      </p>

      <div className="form-row">
        <label htmlFor="price-game">Game</label>
        <select id="price-game" value={game} onChange={(e) => setGame(e.target.value as Game)}>
          <option value="mtg">{GAME_DISPLAY_NAME.mtg}</option>
          <option value="pokemon">{GAME_DISPLAY_NAME.pokemon}</option>
        </select>
      </div>

      <div className="form-row">
        <label htmlFor="price-search">Card name</label>
        <CardSearch game={game} onSelect={handleSelect} id="price-search" />
      </div>

      {loading && <p role="status">Looking up…</p>}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}

      {card && (
        <article className="card-detail">
          <h2>{card.name}</h2>
          <p className="muted">
            {card.set_name} ({card.set_code.toUpperCase()}) · #{card.collector_number}
            {card.rarity ? ` · ${card.rarity}` : ""}
          </p>
          {card.image_url && (
            <img
              src={card.image_url}
              alt={`Card art for ${card.name}`}
              loading="lazy"
              className="card-image"
            />
          )}
          <div className="form-row">
            <button data-variant="primary" disabled={refreshing} onClick={handleRefresh}>
              {refreshing ? "Refreshing…" : "Refresh price"}
            </button>
          </div>
        </article>
      )}

      {prices.length > 0 && (
        <table className="prices-table">
          <thead>
            <tr>
              <th scope="col">Source</th>
              <th scope="col">Currency</th>
              <th scope="col">Price</th>
              <th scope="col">Foil</th>
              <th scope="col">Fetched</th>
            </tr>
          </thead>
          <tbody>
            {prices.map((p, i) => (
              <tr key={`${p.source}-${p.currency}-${p.foil}-${i}`}>
                <td>{p.source}</td>
                <td>{p.currency}</td>
                <td>
                  {(p.cents / 100).toLocaleString(undefined, {
                    style: "currency",
                    currency: p.currency,
                  })}
                </td>
                <td>{p.foil ? "yes" : ""}</td>
                <td>
                  <time dateTime={p.fetched_at}>{p.fetched_at}</time>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {card && prices.length === 0 && !loading && !error && (
        <p className="muted">
          No cached prices yet. Click &quot;Refresh price&quot; to fetch the latest prices.
        </p>
      )}
    </section>
  );
}
