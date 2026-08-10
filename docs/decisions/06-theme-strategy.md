# ADR 006: Theming and Styling Strategy

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-08-09 |
| **Deciders** | Migration Team |
| **Supersedes** | — |
| **Related** | [05-migration-approach.md](05-migration-approach.md) |

---

## Context

OpenTubeX carries a large, hand-maintained stylesheet layer:

| File | Size | Contents |
|------|------|----------|
| `App.css` | ~10,000 lines | Global layout, component styles, resets, utility classes |
| `themes.css` | ~56,000 lines | Named themes, each redefining a large palette of CSS custom properties |

Roughly **66,000 lines of CSS** — comparable in size to the entire Rust backend being written.

Characteristics of the existing system:

- Themes are expressed as large blocks of CSS variables, largely duplicated per theme with only
  colour values differing.
- Selectors are tightly coupled to a bespoke component tree that no longer exists in Slytube.
- Specificity is managed by convention and `!important`; there is no layering discipline.
- No design tokens in the modern sense — variable names encode usage sites, not semantics.
- Light/dark handling predates `prefers-color-scheme` conventions and is bolted on.

Slytube starts from a **fresh Vite + Vue 3 + Tailwind v4 scaffold** with **shadcn-vue already
configured** (`components.json` present; `shadcn-vue`, `reka-ui`, `tailwindcss`,
`class-variance-authority`, `tailwind-merge`, `tw-animate-css` already installed). Per ADR 005,
this is a Big Bang rewrite — the component tree is being rebuilt regardless.

The question: what is the styling baseline for the new component tree?

---

## Options Considered

### Option A — Adopt the shadcn-vue Default Theme (New York)

Use shadcn-vue's default token set and the New York style variant unmodified. Build every
surface from shadcn primitives. Discard `App.css` and `themes.css` entirely.

| Pros | Cons |
|------|------|
| Zero styling migration cost — 66K lines dropped, not ported | Visual identity changes; the app will not look like OpenTubeX |
| shadcn-vue is already installed and configured | Existing users experience a discontinuity |
| Coherent, accessible, well-tested defaults out of the box | Named legacy themes are lost |
| Semantic tokens (`--background`, `--primary`, `--muted-foreground`) | Constrained to shadcn's aesthetic vocabulary initially |
| Dark mode works via a single `.dark` class | |
| Reka UI primitives bring keyboard nav + ARIA for free | |
| Future custom themes are just token overrides — cheap to add later | |

### Option B — Custom shadcn Theme Reproducing OpenTubeX

Keep shadcn-vue's component architecture but redefine the token layer to match OpenTubeX's
palettes, and port the named themes from `themes.css`.

| Pros | Cons |
|------|------|
| Visual continuity for existing users | Requires auditing 56K lines to extract each theme's real palette |
| Named themes survive the migration | Legacy palettes were designed for a different component set — they will not map cleanly |
| Still benefits from shadcn structure | Every shadcn component needs verification against every ported theme |
| | Contrast/accessibility must be re-validated per theme |
| | Substantial, open-ended work with no functional payoff |
| | Bakes legacy colour decisions into a fresh codebase |

### Option C — Hybrid: shadcn Primitives + Retained Legacy CSS

Use shadcn components for new surfaces while keeping `App.css`/`themes.css` for ported views.

| Pros | Cons |
|------|------|
| Incremental — port views at whatever pace suits | **Two competing styling systems in one app** |
| Some visual continuity | Specificity wars between global CSS and Tailwind utilities |
| | Tailwind v4's `@layer` model conflicts with unlayered legacy CSS |
| | 66K lines of dead-ish CSS shipped indefinitely |
| | "Temporary" hybrids are permanent in practice |
| | Every component ambiguous: which system owns it? |

---

## Decision

**Adopt Option A — the shadcn-vue default theme (New York style).**

`App.css` and `themes.css` are **not ported**. Slytube's visual layer is built from shadcn-vue
primitives on Tailwind v4, using shadcn's default semantic token set. Named legacy themes are
dropped; light and dark are the shipping themes at cutover.

### Baseline configuration

```jsonc
// components.json
{
  "style": "new-york",
  "tailwind": { "cssVariables": true, "baseColor": "neutral" },
  "aliases": { "components": "@/components", "utils": "@/lib/utils" }
}
```

```css
/* src/style.css — Tailwind v4, tokens as CSS variables */
@import "tailwindcss";
@import "tw-animate-css";

@custom-variant dark (&:is(.dark *));

:root {
  --background: …;  --foreground: …;
  --primary: …;     --primary-foreground: …;
  --muted: …;       --muted-foreground: …;
  --border: …;      --ring: …;
  --radius: 0.5rem;
}
.dark { /* same tokens, dark values */ }

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  /* … maps tokens into Tailwind's colour scale */
}
```

Components are added via the CLI and **vendored into the repo** (`src/components/ui/`), which is
shadcn's model — they are our source, editable in place, not an opaque dependency.

---

## Rationale

1. **Fresh start.** Per ADR 005 the component tree is being rebuilt from scratch. The legacy CSS
   targets a DOM structure that will not exist. Porting it means rewriting it anyway — but
   constrained by decisions made for a different application. Starting clean is strictly less
   work and yields a better result.

2. **shadcn-vue is already set up.** The scaffold ships with `components.json`, `shadcn-vue`,
   `reka-ui`, Tailwind v4, `class-variance-authority`, and `tailwind-merge` installed. Option A
   is the path of least resistance from the current state; Options B and C both require
   *undoing* or *fighting* that setup.

3. **Dropping legacy colours removes 66K lines of liability.** That CSS is unowned, untested,
   and coupled to a dead component tree. In a migration where the schedule risk is concentrated
   in the Rust backend (ADR 005), spending weeks reverse-engineering theme palettes is
   indefensible. Deleting them is the single largest scope reduction available.

4. **Semantic tokens make future theming cheap.** The reason legacy theming cost 56K lines is
   that it had no token abstraction — each theme restated everything. shadcn's model means a new
   theme is a `:root`/`.dark` variable block, typically under 40 lines. Users who want custom
   themes are better served *after* this migration than by porting the old ones now.

5. **Accessibility and consistency by default.** Reka UI primitives provide focus management,
   keyboard navigation, and ARIA semantics that the bespoke components did not. New York style
   ships with validated contrast ratios. This is quality we get for free and would otherwise
   have to build.

6. **Hybrid is a trap.** Option C's two-system coexistence produces specificity conflicts with
   Tailwind v4's `@layer` model and leaves permanent ambiguity about which system owns a given
   element. The "temporary" legacy layer would never be removed.

---

## Implications

### Component discipline

- [ ] **All components use shadcn primitives.** Buttons, dialogs, dropdowns, tooltips, tabs,
      sheets, popovers, toasts, forms come from `src/components/ui/`. No hand-rolled equivalents.
- [ ] Application components compose primitives; they do not reimplement them.
- [ ] Variants are expressed with `class-variance-authority`, not conditional class strings.
- [ ] Class merging always goes through `cn()` (`clsx` + `tailwind-merge`) so consumer overrides
      win predictably.
- [ ] Genuinely novel surfaces (the video player chrome, the timeline scrubber) are built with
      Tailwind utilities against the same token set — never with a parallel CSS file.

### Tailwind configuration for theming

- [ ] Tokens are defined once as CSS variables in `src/style.css` and mapped into Tailwind via
      `@theme inline`. **Never hard-code a hex value in a component.**
- [ ] Dark mode via the `.dark` class on `<html>`, toggled from a Pinia store and persisted
      through the settings command; honour `prefers-color-scheme` on first run.
- [ ] `--radius` and spacing scale are tokens, so density/roundness are tunable globally.
- [ ] Any future custom theme is an additional token block — the architecture must not require
      touching component code to add one.
- [ ] Keep the token surface small and semantic (`--muted-foreground`, not `--sidebar-item-hover`).
      Usage-site-named variables are exactly how `themes.css` reached 56,000 lines.

### Consequences to accept

| Area | Consequence |
|------|-------------|
| **Visual discontinuity** | Slytube will not look like OpenTubeX. This is a deliberate, communicated change — mention it in release notes rather than letting users discover it. |
| **Named themes removed** | Legacy themes do not survive the cutover. Track community-requested themes as post-migration work; they are cheap to add under the new model. |
| **Design consistency required** | Without a designer, the app inherits shadcn's aesthetic. That is a coherent baseline — resist ad-hoc deviation from it, since inconsistency is worse than a plain-but-uniform UI. |
| **Vendored components** | `src/components/ui/` is our code. Upstream shadcn-vue fixes are not automatic; note any local modifications so they can be reconciled on manual updates. |
| **Tailwind v4 specifics** | v4 uses CSS-first config (`@theme`) rather than `tailwind.config.js`. Contributors familiar with v3 need to be pointed at this. |
| **No global CSS escape hatch** | There is no `App.css` to reach for. Cross-cutting styling belongs in tokens or a shared component, and this must be enforced in review. |

---

## References

- Legacy baseline: `App.css` (~10K lines), `themes.css` (~56K lines)
- [shadcn-vue documentation](https://www.shadcn-vue.com/) · [Theming](https://www.shadcn-vue.com/docs/theming)
- [Reka UI](https://reka-ui.com/)
- [Tailwind CSS v4 — CSS-first configuration](https://tailwindcss.com/docs/theme)
- [05-migration-approach.md](05-migration-approach.md) §"Scope discipline"
