// SPDX-License-Identifier: AGPL-3.0-or-later
import { useCallback, useEffect, useRef, useState } from "react";
import type { KeyboardEvent, ReactElement } from "react";
import { api } from "../lib/tauri";
import type { Card, Game } from "../types";

interface CardSearchProps {
  game: Game;
  /** Called when the user picks a result. */
  onSelect: (card: Card) => void;
  /** Placeholder text for the input. */
  placeholder?: string;
  id?: string;
}

/**
 * Debounced card-name autocomplete backed by `catalog_search`.
 * Displays a dropdown of matches; selecting one calls `onSelect`.
 */
export function CardSearch({ game, onSelect, placeholder, id }: CardSearchProps): ReactElement {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Card[]>([]);
  const [open, setOpen] = useState(false);
  const [activeIdx, setActiveIdx] = useState(-1);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Close dropdown on outside click.
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  const doSearch = useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (trimmed.length < 2) {
        setResults([]);
        setOpen(false);
        return;
      }
      try {
        const cards = await api.catalog.search(trimmed, { game, limit: 10 });
        setResults(cards);
        setOpen(cards.length > 0);
        setActiveIdx(-1);
      } catch {
        setResults([]);
        setOpen(false);
      }
    },
    [game],
  );

  function handleChange(value: string) {
    setQuery(value);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => doSearch(value), 250);
  }

  // Clean up pending timer on unmount.
  useEffect(() => {
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, []);

  function pick(card: Card) {
    setQuery(card.name);
    setOpen(false);
    setResults([]);
    onSelect(card);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!open || results.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIdx((i) => (i + 1) % results.length);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIdx((i) => (i <= 0 ? results.length - 1 : i - 1));
    } else if (e.key === "Enter" && activeIdx >= 0) {
      e.preventDefault();
      pick(results[activeIdx]);
    } else if (e.key === "Escape") {
      setOpen(false);
    }
  }

  return (
    <div className="card-search" ref={containerRef}>
      <input
        id={id}
        type="text"
        value={query}
        onChange={(e) => handleChange(e.target.value)}
        onFocus={() => results.length > 0 && setOpen(true)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder ?? "Search by card name\u2026"}
        autoComplete="off"
        role="combobox"
        aria-expanded={open}
        aria-autocomplete="list"
        aria-controls={id ? `${id}-listbox` : undefined}
        aria-activedescendant={activeIdx >= 0 ? `card-opt-${activeIdx}` : undefined}
      />
      {open && results.length > 0 && (
        <ul className="card-search__results" role="listbox" id={id ? `${id}-listbox` : undefined}>
          {results.map((card, i) => (
            <li
              key={`${card.game}-${card.id}`}
              id={`card-opt-${i}`}
              role="option"
              aria-selected={i === activeIdx}
              className={`card-search__item${i === activeIdx ? " card-search__item--active" : ""}`}
              onMouseDown={() => pick(card)}
            >
              <span className="card-search__name">{card.name}</span>
              <span className="card-search__meta muted">
                {card.set_code.toUpperCase()} #{card.collector_number}
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
