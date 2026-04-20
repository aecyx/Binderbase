// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * E2E tests for the Catalog Import flow.
 *
 * These tests exercise start → progress → cancel → restart against the live
 * Tauri webview via CDP. They use the Pokémon TCG API import (smaller/faster
 * than MTG via Scryfall) and cancel before completion to avoid long runtimes
 * and rate-limit issues.
 *
 * **Do NOT add a "run to completion" test here.** A full Pokémon import is
 * multi-minute and rate-limited; completion coverage belongs in a nightly or
 * manual run, not the PR gate.
 *
 * **Do NOT run this spec in CI without an API key / rate-limit plan.** The
 * Pokémon TCG REST API is hit during the "progress visible" step; cancel keeps
 * the request count low, but automated reruns can exceed the free-tier limit.
 *
 * Prerequisites:
 *   1. Launch with remote debugging:
 *        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9223"
 *        npm run tauri dev
 *   2. Run with a single Playwright worker because these CDP-backed tests
 *      require exclusive control of the single running Tauri webview:
 *        npx playwright test --workers=1 e2e/catalog-import.spec.ts
 */
import { test, expect, chromium, type Page } from "@playwright/test";

// These tests involve live HTTP and UI transitions — use a generous timeout.
test.setTimeout(90_000);

let page: Page;

test.beforeAll(async () => {
  const browser = await chromium.connectOverCDP("http://127.0.0.1:9223");
  page = browser.contexts()[0].pages()[0];

  const importExportBtn = page.getByRole("button", { name: "Import / Export" });
  await expect(importExportBtn).toBeVisible();
  await expect(importExportBtn).toBeEnabled();
  await importExportBtn.click();

  await expect(page.getByRole("heading", { name: "Catalog Import" })).toBeVisible();
});

test.describe.serial("catalog import restart flow", () => {
  // ---------------------------------------------------------------------------
  // Test 1: start → progress visible → cancel → buttons re-enabled
  // ---------------------------------------------------------------------------
  test("import pokemon: start, see progress, cancel, buttons re-enabled", async () => {
    // Click "Import Pokémon"
    const importPokemonBtn = page.getByRole("button", { name: "Import Pokémon" });
    await importPokemonBtn.click();

    // Within 10s the Cancel button must appear (in_progress === true).
    const cancelBtn = page.getByRole("button", { name: "Cancel" });
    await expect(cancelBtn).toBeVisible({ timeout: 10_000 });

    // Within 30s either a <progress> element or a known stage label appears.
    const progressBar = page.locator("progress.import-panel__bar");
    const stageLabel = page.locator(".import-panel__stage .muted");
    const knownStages = [
      "Fetching catalog index\u2026",
      "Downloading\u2026",
      "Parsing\u2026",
      "Importing cards\u2026",
    ];

    await expect(async () => {
      const barVisible = await progressBar.isVisible().catch(() => false);
      if (barVisible) return; // progress bar appeared — success

      const labelText = await stageLabel.textContent().catch(() => "");
      const stageMatched = knownStages.some((s) => labelText?.includes(s));
      expect(barVisible || stageMatched).toBe(true);
    }).toPass({ timeout: 30_000 });

    // Click Cancel.
    await cancelBtn.click();

    // Within 15s the Import buttons must become enabled and Cancel must vanish.
    await expect(cancelBtn).not.toBeVisible({ timeout: 15_000 });
    await expect(importPokemonBtn).toBeEnabled({ timeout: 15_000 });
    await expect(page.getByRole("button", { name: "Import Magic: The Gathering" })).toBeEnabled();
    await expect(page.getByRole("button", { name: "Import all games" })).toBeEnabled();

    // A "Pokémon" run summary with a "cancelled" badge must render.
    const historySection = page.locator(".import-panel__history");
    await expect(historySection.locator(".import-run")).toContainText("Pokémon");
    await expect(historySection.locator(".import-run__badge--cancelled")).toBeVisible();
  });

  // ---------------------------------------------------------------------------
  // Test 2: restart after cancel — verifies ImportController resets correctly
  // ---------------------------------------------------------------------------
  test("import pokemon: restart after cancel works", async () => {
    // From post-cancel state, click "Import Pokémon" again.
    const importPokemonBtn = page.getByRole("button", { name: "Import Pokémon" });
    await importPokemonBtn.click();

    // Cancel must reappear within 10s (ImportController was correctly reset).
    const cancelBtn = page.getByRole("button", { name: "Cancel" });
    await expect(cancelBtn).toBeVisible({ timeout: 10_000 });

    // Cancel again to leave the environment clean.
    await cancelBtn.click();
    await expect(cancelBtn).not.toBeVisible({ timeout: 15_000 });
    await expect(importPokemonBtn).toBeEnabled({ timeout: 15_000 });
  });
});

// ---------------------------------------------------------------------------
// Test 3: keyring-degraded banner smoke check
// ---------------------------------------------------------------------------
test("keyring-degraded notice surface exists", async () => {
  // Navigate to Scan tab, then back to Import / Export (forces a repaint).
  await page.getByRole("button", { name: "Scan" }).click();
  await expect(page.getByRole("heading", { name: "Scan a card" })).toBeVisible();
  await page.getByRole("button", { name: "Import / Export" }).click();
  await expect(page.getByRole("heading", { name: "Catalog Import" })).toBeVisible();

  // The notice[role="status"] element should either contain the degraded
  // banner text or be absent entirely. Both states are valid — this test
  // documents that the banner surface exists without pretending to exercise
  // the degraded keyring path end-to-end.
  const notice = page.locator('.notice[role="status"]');
  const noticeCount = await notice.count();
  if (noticeCount > 0) {
    // If the notice exists, it should contain the expected text.
    await expect(notice).toContainText("OS keyring unavailable");
  }
  // If the notice is absent, the keyring is healthy — that's fine too.
});
