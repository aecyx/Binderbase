// SPDX-License-Identifier: AGPL-3.0-or-later
import { useState } from "react";
import type { ReactElement } from "react";
import { api } from "../../lib/tauri";
import type { Card, Game, Price } from "../../types";
import { GAME_DISPLAY_NAME, isBinderbaseError } from "../../types";

export function PricesPage(): ReactElement {
  const [game, setGame] = useState<Game>("mtg");
  const [cardId, setCardId] = useState("");
  const [card, setCard] = useState<Card | null>(null);
  const [prices, setPrices] = useState<Price[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function lookup() {
    if (!cardId.trim()) return;
    setError(null);
    setCard(null);
    setPrices([]);
    setLoading(true);
    try {
      const [c, p] = await Promise.all([
        api.fetchCard(game, cardId.trim()),
        api.pricing.getCached(game, cardId.trim()),
      ]);
      setCard(c);
      setPrices(p);
    } catch (e) {
      setError(isBinderbaseError(e) ? `${e.kind}: ${e.message}` : String(e));
    } finally {
      setLoading(false);
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
        <label htmlFor="card-id-input">Card id</label>
        <input
          id="card-id-input"
          value={cardId}
          onChange={(e) => setCardId(e.target.value)}
          placeholder={
            game === "mtg"
              ? "Scryfall UUID, e.g. 0000579f-7b35-4ed3-b44c-db2a538066fe"
              : "PTCGAPI id, e.g. swsh4-25"
          }
        />
        <button
          data-variant="primary"
          type="button"
          onClick={lookup}
          disabled={loading || !cardId.trim()}
        >
          Look up
        </button>
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
          No cached prices yet. Price refresh is not wired in 0.1 — see roadmap.
        </p>
      )}
    </section>
  );
}
