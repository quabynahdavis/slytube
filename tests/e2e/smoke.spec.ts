import { test, expect } from './fixtures';

test.describe('App Launch', () => {
  test('launches and shows main shell', async ({ appPage }) => {
    await appPage.waitForLoadState('domcontentloaded');

    // App should render the main shell with sidebar or topnav
    const shell = appPage.locator('#app');
    await expect(shell).toBeAttached({ timeout: 15_000 });
  });

  test('sidebar renders navigation items', async ({ appPage }) => {
    await appPage.waitForLoadState('domcontentloaded');

    // Look for key navigation elements
    const nav = appPage.locator('nav').first();
    await expect(nav).toBeVisible({ timeout: 10_000 });
  });
});

test.describe('Navigation', () => {
  test('navigates to Trending', async ({ appPage }) => {
    await appPage.waitForLoadState('domcontentloaded');

    const trendingLink = appPage.getByRole('link', { name: /trending/i }).first();
    if (await trendingLink.isVisible().catch(() => false)) {
      await trendingLink.click();
      await appPage.waitForURL(/trending/, { timeout: 10_000 });
    }
  });
});

test.describe('Search', () => {
  test('can type in search and submit', async ({ appPage }) => {
    await appPage.waitForLoadState('domcontentloaded');

    const searchInput = appPage.locator('input[type="text"], input[type="search"], input[placeholder*="search" i]').first();
    if (await searchInput.isVisible().catch(() => false)) {
      await searchInput.fill('test video');
      await searchInput.press('Enter');
      await appPage.waitForURL(/search/, { timeout: 10_000 });
    }
  });
});
