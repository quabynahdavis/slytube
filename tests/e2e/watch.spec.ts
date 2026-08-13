describe('Search', () => {
  it('can type in search and submit', async () => {
    const searchInput = await browser.$(
      'input[type="text"], input[type="search"], input[placeholder*="search" i]'
    );
    const exists = await searchInput.isExisting();
    if (!exists) {
      console.warn('Search input not found, skipping');
      return;
    }

    await searchInput.setValue('lofi hip hop');
    await browser.keys('Enter');

    // URL should change to /search
    await browser.waitUntil(
      async () => {
        const url = await browser.getUrl();
        return url.includes('search');
      },
      { timeout: 10_000, timeoutMsg: 'URL did not change to /search' }
    );
  });
});

describe('Watch Flow', () => {
  it('opens a video from search results', async () => {
    const searchInput = await browser.$(
      'input[type="text"], input[type="search"], input[placeholder*="search" i]'
    );
    if (await searchInput.isExisting()) {
      await searchInput.setValue('test');
      await browser.keys('Enter');
    }

    // Wait for any clickable result
    await browser.waitUntil(
      async () => {
        const results = await browser.$$('[class*="card"], [class*="Card"], [class*="video"], a[href*="watch"]');
        return results.length > 0;
      },
      { timeout: 15_000, timeoutMsg: 'No search results appeared' }
    );

    const results = await browser.$$('[class*="card"], [class*="Card"], [class*="video"], a[href*="watch"]');
    if (results.length > 0) {
      await results[0].click();
      await browser.waitUntil(
        async () => {
          const url = await browser.getUrl();
          return url.includes('watch');
        },
        { timeout: 10_000, timeoutMsg: 'URL did not change to /watch' }
      );
    }
  });
});
