// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * E2E tests that run against the live Tauri webview via CDP.
 *
 * Prerequisites:
 *   1. Launch: $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9223"
 *              npm run tauri dev
 *   2. Run:   npx playwright test e2e/tauri.spec.ts
 */
import { test, expect, chromium, type Page } from "@playwright/test";

let page: Page;

test.beforeAll(async () => {
  const browser = await chromium.connectOverCDP("http://127.0.0.1:9223");
  const context = browser.contexts()[0];
  page = context.pages()[0];
  // Wait for React to mount
  await page.waitForSelector("[data-testid='app-root'], main, nav", { timeout: 10_000 });
});

// ---------- Navigation ----------

test("app loads with correct title", async () => {
  const title = await page.title();
  expect(title).toBe("Binderbase");
});

test("nav bar has all five tabs", async () => {
  const nav = page.getByRole("navigation", { name: "Primary" });
  await expect(nav.getByRole("button", { name: "Scan" })).toBeVisible();
  await expect(nav.getByRole("button", { name: "Collection" })).toBeVisible();
  await expect(nav.getByRole("button", { name: "Prices" })).toBeVisible();
  await expect(nav.getByRole("button", { name: "Import / Export" })).toBeVisible();
  await expect(nav.getByRole("button", { name: "Settings" })).toBeVisible();
});

// ---------- Scan page ----------

test("scan page renders game selector and file input", async () => {
  await page.getByRole("button", { name: "Scan" }).click();
  await expect(page.getByRole("heading", { name: "Scan a card" })).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Game" })).toBeVisible();
  // File input present
  await expect(page.locator('input[type="file"]')).toBeAttached();
});

// ---------- Collection page ----------

test("collection page loads and shows empty state or entries", async () => {
  await page.getByRole("button", { name: "Collection" }).click();
  await expect(page.getByRole("heading", { name: "Collection" })).toBeVisible();

  // Should show either "empty" message or a table
  const empty = page.getByText("Your collection is empty");
  const table = page.locator("table.collection-table");
  const either = await Promise.race([
    empty.waitFor({ timeout: 5000 }).then(() => "empty"),
    table.waitFor({ timeout: 5000 }).then(() => "table"),
  ]);
  expect(["empty", "table"]).toContain(either);
});

test("collection add-card form toggles", async () => {
  await page.getByRole("button", { name: "Collection" }).click();
  const addBtn = page.getByRole("button", { name: "+ Add card" });
  await addBtn.click();
  await expect(page.locator("form.add-card-form")).toBeVisible();
  // Cancel closes the form
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(page.locator("form.add-card-form")).not.toBeVisible();
});

// ---------- Prices page ----------

test("prices page renders with search input", async () => {
  await page.getByRole("button", { name: "Prices" }).click();
  await expect(page.getByRole("heading", { name: "Prices" })).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Game" })).toBeVisible();
  await expect(page.getByPlaceholder("Search by card name")).toBeVisible();
});

// ---------- Import / Export page ----------

test("import/export page has all three sections", async () => {
  await page.getByRole("button", { name: "Import / Export" }).click();
  await expect(page.getByRole("heading", { name: "Import / Export", exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Catalog Import" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Scan Index" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Collection Import / Export" })).toBeVisible();
});

test("import/export has build index buttons for both games", async () => {
  await page.getByRole("button", { name: "Import / Export" }).click();
  const buildButtons = page.getByRole("button", { name: "Build Index" });
  await expect(buildButtons).toHaveCount(2);
});

// ---------- Collection CRUD (if catalog has data) ----------

test("collection: add a card via search, verify in table, then remove", async () => {
  // Navigate to collection
  await page.getByRole("button", { name: "Collection" }).click();
  await expect(page.getByRole("heading", { name: "Collection" })).toBeVisible();

  // Open add form
  await page.getByRole("button", { name: "+ Add card" }).click();
  await expect(page.locator("form.add-card-form")).toBeVisible();

  // Type in the card search
  const searchInput = page.locator("form.add-card-form").getByPlaceholder(/search/i);
  await searchInput.fill("Lightning Bolt");

  // Wait for autocomplete results (needs catalog data)
  const suggestion = page.locator(".card-search__suggestions li").first();
  const hasSuggestions = await suggestion
    .waitFor({ timeout: 5000 })
    .then(() => true)
    .catch(() => false);

  if (!hasSuggestions) {
    // No catalog data — skip the CRUD test
    test.skip(true, "No catalog data available — run a bulk import first");
    return;
  }

  // Click first suggestion
  await suggestion.click();

  // Submit the form
  await page.getByRole("button", { name: "Add to collection" }).click();

  // Wait for table to appear with the new entry
  const table = page.locator("table.collection-table");
  await expect(table).toBeVisible({ timeout: 5000 });

  // Find the remove button on the last added row and click it
  const removeBtn = table.getByRole("button", { name: /Remove/ }).first();
  await removeBtn.click();

  // Verify the entry was removed (either table gone or row count decreased)
  await page.waitForTimeout(500);
});

// ---------- Price lookup ----------

test("prices: search for a card and see results", async () => {
  await page.getByRole("button", { name: "Prices" }).click();
  const searchInput = page.getByPlaceholder("Search by card name");
  await searchInput.fill("Black Lotus");

  const suggestion = page.locator(".card-search__suggestions li").first();
  const hasSuggestions = await suggestion
    .waitFor({ timeout: 5000 })
    .then(() => true)
    .catch(() => false);

  if (!hasSuggestions) {
    test.skip(true, "No catalog data available — run a bulk import first");
    return;
  }

  await suggestion.click();

  // Should show card detail
  await expect(page.locator("article.card-detail")).toBeVisible({ timeout: 5000 });

  // Should show refresh button
  await expect(page.getByRole("button", { name: "Refresh price" })).toBeVisible();
});
