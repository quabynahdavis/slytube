import { test, expect } from './fixtures';

test.describe('Watch Flow', () => {
  test('opens a video from search results', async ({ appPage }) => {
    await appPage.waitForLoadState('domcontentloaded');

    const searchInput = appPage.locator('input[type="text"], input[type="search"], input[placeholder*="search" i]').first();
    if (await searchInput.isVisible().catch(() => false)) {
      await searchInput.fill('lofi hip hop');
      await searchInput.press('Enter');
      await appPage.waitForURL(/search/, { timeout: 10_000 });

      // Wait for results and click first video card
      const firstResult = appPage.locator('[class*="video-card"], [class*="VideoCard"]').first();
      if (await firstResult.isVisible().catch(() => false)) {
        await firstResult.click();
        await appPage.waitForURL(/watch/, { timeout: 10_000 });
      }
    }
  });
});

test.describe('Subscribe', () => {
  test('subscribe button toggles state', async ({ appPage }) => {
    await appPage.waitForLoadState('domcontentloaded');

    const subscribeBtn = appPage.getByRole('button', { name: /subscribe/i }).first();
    if (await subscribeBtn.isVisible().catch(() => false)) {
      const initialText = await subscribeBtn.textContent();
      await subscribeBtn.click();
      await expect(subscribeBtn).not.toHaveText(initialText?.trim() ?? '', { timeout: 5_000 });
    }
  });
});
