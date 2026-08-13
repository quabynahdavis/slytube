describe('App Launch', () => {
  it('launches and shows main shell', async () => {
    await browser.waitUntil(
      async () => {
        const app = await browser.$('#app');
        return app.isExisting();
      },
      { timeout: 15_000, timeoutMsg: 'App shell (#app) never appeared' }
    );
  });

  it('renders sidebar navigation', async () => {
    await browser.waitUntil(
      async () => {
        const nav = await browser.$$('nav');
        return nav.length > 0;
      },
      { timeout: 10_000, timeoutMsg: 'No <nav> element found' }
    );
  });
});

describe('Navigation', () => {
  it('has a link or button to Trending', async () => {
    await browser.waitUntil(
      async () => {
        const els = await browser.$$('a, button');
        for (const el of els) {
          const text = await el.getText();
          if (/trending/i.test(text)) return true;
        }
        return false;
      },
      { timeout: 10_000, timeoutMsg: 'No Trending link found' }
    );
  });
});
