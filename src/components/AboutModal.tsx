// SPDX-License-Identifier: AGPL-3.0-or-later
import { useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";

interface Props {
  version?: string;
}

export function AboutModal({ version }: Props): ReactElement {
  const [open, setOpen] = useState(false);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    if (open) {
      dialogRef.current?.showModal();
    } else {
      dialogRef.current?.close();
    }
  }, [open]);

  return (
    <>
      <button type="button" className="about-trigger" onClick={() => setOpen(true)}>
        About
      </button>
      <dialog ref={dialogRef} className="about-dialog" onClose={() => setOpen(false)}>
        <h2>Binderbase{version ? ` v${version}` : ""}</h2>
        <p>A local-first trading-card-game scanner and collection manager.</p>

        <h3>Data sources</h3>
        <ul className="about-credits">
          <li>
            Card data for Magic: The Gathering provided by{" "}
            <a href="https://scryfall.com" target="_blank" rel="noopener noreferrer">
              Scryfall
            </a>
            . Binderbase is not produced by or endorsed by Scryfall.
          </li>
          <li>
            Card data for Pokémon TCG provided by the{" "}
            <a href="https://pokemontcg.io" target="_blank" rel="noopener noreferrer">
              Pokémon TCG API
            </a>
            . Binderbase is not produced by or endorsed by Pokémon TCG API.
          </li>
        </ul>

        <h3>Privacy</h3>
        <p>
          Binderbase does not collect or transmit any user data. The only network calls are to
          Scryfall and the Pokémon TCG API to fetch card data and prices. All collection data is
          stored locally on your machine.
        </p>

        <h3>License</h3>
        <p>
          <a
            href="https://github.com/aecyx/Binderbase/blob/main/LICENSE"
            target="_blank"
            rel="noopener noreferrer"
          >
            AGPL-3.0-or-later
          </a>
        </p>

        {version && (
          <p className="muted">
            <a
              href="https://github.com/aecyx/Binderbase/releases"
              target="_blank"
              rel="noopener noreferrer"
            >
              Check for updates
            </a>
          </p>
        )}

        <div className="about-dialog__footer">
          <button type="button" data-variant="primary" onClick={() => setOpen(false)}>
            Close
          </button>
        </div>
      </dialog>
    </>
  );
}
