# Alex Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor goose2 UI to match designer Alex's Q2 2026 Figma — restrained, editorial, Cash Sans, no shadows, single-weight hierarchy, static decorative placeholders for unbuilt features.

**Architecture:** Five sequential commits on `tulsi/visual-design`, cut from `origin/main`. Commit 1 lands foundation (tokens, fonts, asset downloads, shell, GlobalComposerPill). Commits 2-5 re-skin Chat, Skills, Agents, Sessions in order, each gated by a dev-server visual check.

**Tech Stack:** Tauri v2 + React 18 + TypeScript + Vite + Tailwind (v4 with `@theme inline`) + Zustand stores + shadcn/ui-derived components + Cash Sans (self-hosted from Block CDN) + Figma MCP for asset extraction.

**Source of truth for design:** `docs/superpowers/specs/2026-04-23-alex-redesign-design.md` — read that first. This plan is the execution sequence against it.

---

## File structure overview

```
ui/goose2/
├── docs/superpowers/
│   ├── plans/2026-04-23-alex-redesign.md          [this file]
│   └── specs/2026-04-23-alex-redesign-design.md   [design spec]
├── src/
│   ├── app/
│   │   ├── AppShell.tsx                           MODIFY (bg-dot-grid, pill mount)
│   │   └── ui/
│   │       ├── AppShellContent.tsx                MODIFY (route home → HomeView)
│   │       └── TopBar.tsx                         REWRITE (breadcrumb, Settings pill)
│   ├── features/
│   │   ├── agents/ui/
│   │   │   ├── PersonaCard.tsx                    REWRITE body (single figure + rule + pill + desc)
│   │   │   └── PersonaGallery.tsx                 MODIFY grid template
│   │   ├── chat/ui/
│   │   │   ├── ChatView.tsx                       MODIFY (wrap in bounded card)
│   │   │   └── ChatInput.tsx                      MODIFY outer container
│   │   ├── home/ui/
│   │   │   └── HomeView.tsx                       CREATE (static widget placeholders)
│   │   ├── sessions/ui/
│   │   │   ├── SessionCard.tsx                    MODIFY (surface-less re-skin)
│   │   │   └── SessionHistoryView.tsx             MODIFY (page polish, grouped headers)
│   │   ├── sidebar/ui/
│   │   │   └── Sidebar.tsx                        MODIFY (token re-skin, preserve IA)
│   │   └── skills/ui/
│   │       ├── CategoryHeroTile.tsx               CREATE (static decorative tile)
│   │       └── SkillsView.tsx                     MODIFY (categories, cards, toolbar)
│   ├── shared/
│   │   ├── styles/globals.css                     MODIFY (all token additions)
│   │   └── ui/GlobalComposerPill.tsx              CREATE (floating pill, not on Chat)
│   └── assets/                                    CREATE new directories + files
│       ├── agents/                                1 shared figure (Figma uses single cutout)
│       ├── home/                                  world cube, clock, figure (reused), sticky note
│       └── skills/                                6 category photos + 1 mask (Research = CSS gradient, see §10.1)
```

**Total:** ~14 modified/created source files + 1 new assets folder tree + 1 plan doc (this file) committed with the spec in Commit 1.

---

## Pre-flight — Task 0

### Task 0.1: Verify working tree + dev server baseline

**Files:** none (environment check)

- [ ] **Step 1: Confirm branch**

Run: `git branch --show-current`
Expected: `tulsi/visual-design`

- [ ] **Step 2: Confirm working tree state**

Run: `git status --short`
Expected: one staged file `A  ui/goose2/docs/superpowers/specs/2026-04-23-alex-redesign-design.md` and (after this plan is committed-or-staged) possibly `A  ui/goose2/docs/superpowers/plans/2026-04-23-alex-redesign.md`.

If anything else appears, stop and investigate — the stash may have leaked state. Recover with `git diff HEAD` and compare to stash contents via `git stash show -p`.

- [ ] **Step 3: Stage this plan file alongside the spec**

Run: `git add ui/goose2/docs/superpowers/plans/2026-04-23-alex-redesign.md`

- [ ] **Step 4: Typecheck baseline**

Run: `cd ui/goose2 && pnpm typecheck 2>&1 | tail -20`
Expected: 4 errors in `src/features/extensions/api/extensions.ts` and `src/shared/api/acpApi.ts`. No other errors. This is the pre-existing SDK-drift baseline — don't regress beyond it.

- [ ] **Step 5: Dev server starts clean**

Run (in a separate terminal): `cd ui/goose2 && pnpm tauri dev`
Expected: Tauri window opens showing current goose2 UI. Confirm sidebar + topbar + chat view all render. Close when done.

- [ ] **Step 6: Do NOT commit yet**

The spec + plan land as part of Commit 1 at the end of the Foundation phase, not as a standalone commit.

---

## Phase 1 — Foundation (Commit 1)

### Task 1.1: Add all token additions to `globals.css`

**Files:**
- Modify: `ui/goose2/src/shared/styles/globals.css` (append new block below existing tokens)

- [ ] **Step 1: Read current file**

Open `ui/goose2/src/shared/styles/globals.css`. Locate the end of the `:root { ... }` block (likely around line 150 given the existing token density).

- [ ] **Step 2: Add Cash Sans font-face declarations at the top of the file (before any `@import`)**

Exact URLs for the Cash Sans woff2 files must be pulled from Regulator PR 2308 at implementation time:

```bash
gh api 'repos/squareup/regulator-fe/contents/client/src/assets/fonts.css?ref=refs/pull/2308/head' --jq '.content' | base64 -d | head -80
```

Grep the output for `CashSans-Regular` and `CashSans-RegularItalic` woff2 URLs. The base CDN path is `https://cash-f.squarecdn.com/static/fonts/cashsans/v2/`. Copy the two exact filenames into the `@font-face` blocks below.

Insert at the very top of `globals.css`, above the existing `@import "tailwindcss";`:

```css
@font-face {
  font-family: "Cash Sans";
  src: url("https://cash-f.squarecdn.com/static/fonts/cashsans/v2/<REGULAR>.woff2") format("woff2");
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: "Cash Sans";
  src: url("https://cash-f.squarecdn.com/static/fonts/cashsans/v2/<ITALIC>.woff2") format("woff2");
  font-weight: 400;
  font-style: italic;
  font-display: swap;
}
```

Replace `<REGULAR>` and `<ITALIC>` with the actual filenames from the Regulator PR.

- [ ] **Step 3: Add `html { color-scheme: light }` lock**

Append at the end of the file:

```css
html {
  color-scheme: light;
}
```

- [ ] **Step 4: Add token block to `:root`**

Inside the existing `:root { ... }` block (bottom), append:

```css
:root {
  /* Alex redesign — foundation tokens */

  /* canvas + surfaces */
  --canvas:            #dedede;
  --surface-chrome:    rgba(255, 255, 255, 0.5);
  --surface-composer:  rgba(255, 255, 255, 0.2);
  --surface-button:    #f5f5f5;
  --surface-card:      #ffffff;
  --surface-tile:      #f5f5f5;
  --surface-install:   #dedede;

  /* text */
  --text-default-alex: #242424;
  --text-muted-alex:   #7f7f7f;
  --text-title-alex:   #19191a;

  /* tag pills (Skills; cycle by index) */
  --pill-pink:         #eec2ea;
  --pill-olive:        #cdcda1;
  --pill-blue:         #bcc6f4;
  --pill-neutral:      #ffffff;

  /* dot grid */
  --dot-color:         rgba(37, 37, 37, 0.15);
  --dot-size:          1px;
  --dot-spacing:       24px;

  /* typography sizes (role-named) */
  --text-label-alex:   10px;
  --text-body-alex:    14px;
  --text-input-alex:   16px;
  --text-title-size:   24px;

  /* radii */
  --radius-chrome:     16px;
  --radius-composer:   40px;
  --radius-pill:       9999px;
  --radius-tile:       20px;
  --radius-card-chat:  20px;  /* tentative; may update in Task 2.2 after Figma extract */

  /* font */
  --font-sans-alex: "Cash Sans", system-ui, -apple-system, BlinkMacSystemFont,
                    "Segoe UI", Roboto, sans-serif;
}
```

Note the `-alex` suffixes on a few tokens that collide with Ghost's existing semantic names (`--text-default`, `--text-muted`, `--text-title` already exist per the current `globals.css`). Using an explicit suffix avoids breaking components that consume Ghost's tokens.

- [ ] **Step 5: Add dot grid utility**

Append after the `:root` block:

```css
.bg-dot-grid {
  background-color: var(--canvas);
  background-image: radial-gradient(
    circle,
    var(--dot-color) var(--dot-size),
    transparent var(--dot-size)
  );
  background-size: var(--dot-spacing) var(--dot-spacing);
}
```

- [ ] **Step 6: Expose tokens to Tailwind via `@theme inline`**

Locate the existing `@theme inline` block (if any) or add one. Append:

```css
@theme inline {
  --color-canvas-alex:            var(--canvas);
  --color-surface-chrome:         var(--surface-chrome);
  --color-surface-composer:       var(--surface-composer);
  --color-surface-button:         var(--surface-button);
  --color-surface-card:           var(--surface-card);
  --color-surface-tile:           var(--surface-tile);
  --color-surface-install:        var(--surface-install);
  --color-text-default-alex:      var(--text-default-alex);
  --color-text-muted-alex:        var(--text-muted-alex);
  --color-text-title-alex:        var(--text-title-alex);
  --color-pill-pink:              var(--pill-pink);
  --color-pill-olive:             var(--pill-olive);
  --color-pill-blue:              var(--pill-blue);
  --color-pill-neutral:           var(--pill-neutral);

  --radius-chrome:                var(--radius-chrome);
  --radius-composer:              var(--radius-composer);
  --radius-tile:                  var(--radius-tile);
  --radius-card-chat:             var(--radius-card-chat);
}
```

Gives you `bg-canvas-alex`, `bg-surface-chrome`, `text-muted-alex`, `rounded-chrome`, `rounded-composer`, `rounded-tile`, etc. as Tailwind utilities.

- [ ] **Step 7: Apply Cash Sans at the body level**

Locate the `body` rule in `globals.css` (likely in a `@layer base` or raw). Add `font-family: var(--font-sans-alex)`:

```css
@layer base {
  body {
    font-family: var(--font-sans-alex);
    /* ...existing body styles... */
  }
}
```

If a `body { font-family: ... }` already exists, replace its value with `var(--font-sans-alex)`.

- [ ] **Step 8: Run typecheck + lint**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -10
cd ui/goose2 && pnpm lint 2>&1 | tail -10
```

Expected typecheck: 4 baseline errors. Expected lint: pass or same baseline.

- [ ] **Step 9: Start dev server and sanity-check**

```bash
cd ui/goose2 && pnpm tauri dev
```

Expected: nothing broken. Body font should now be Cash Sans (compare to system-ui in prior screenshot). Canvas color unchanged — the `.bg-dot-grid` class is defined but not yet applied anywhere. Sidebar/chat/etc. still look like origin/main. **Do not commit.**

---

### Task 1.2: Download Figma assets

**Files:**
- Create directories: `ui/goose2/src/assets/home/`, `ui/goose2/src/assets/skills/`, `ui/goose2/src/assets/agents/`
- Create files: `home/` has 4 (cube, clock, sticky-note, figure), `skills/` has 7 (6 category photos + 1 mask; Research tile renders a CSS gradient per spec §10.1 fallback pattern — no file), `agents/` has 1 (figure, shared across all personas)

- [ ] **Step 1: Prepare directories**

```bash
mkdir -p ui/goose2/src/assets/home ui/goose2/src/assets/skills ui/goose2/src/assets/agents
```

- [ ] **Step 2: Refresh Figma asset URLs via MCP**

The URLs in the spec are from 2026-04-23 and expire after 7 days. Re-fetch fresh URLs for each source frame using the Figma MCP `get_design_context` tool:

```
fileKey: e43a6gyBVn1SdARFkZpN0N

Frames to inspect:
- 110:66   (Landing)               → world cube + clock + sticky note + single figure (reused at 3 positions)
- 234:2240 (Skills — Full)          → 6 category photos + 260x260 mask (Research skipped — see Step 4 fallback)
- 114:1485 (Agents — More than 5)   → single reused figure asset (see Step 5)
```

For each frame, call `mcp__figma__get_design_context` with `excludeScreenshot: true`. The response will list image asset URLs as `const imgXXX = "https://www.figma.com/api/mcp/asset/<uuid>";` at the top of the generated code. Match each imported image constant to its use in the code (e.g. `imgImage139` shows up as the world-cube background).

- [ ] **Step 3: Identify and download Home widget assets**

From Landing frame (`110:66`), identify:
- `imgImage139`: the "world cube" photo (blurry blue/white; `imgImage140` is the mask shape, applied via `maskImage` — not a composed image and not downloaded)
- `imgImage6090`: the single human figure cutout. The frame re-renders this same asset at 3 positions with different crops. One URL, one file.
- `imgGroup2147229963`: the analog clock composition
- `imgGroup2147229965`: sticky note with red dot (check node 112:1242)

Download each. Commands (substituting the actual URLs from the MCP response):

Figma's asset server returns each export in its native format — the file extension must be chosen after running `file` against the download. On 2026-04-23, `world-cube` and `figure` came back as PNG, `clock` and `sticky-note` came back as SVG. Name the files accordingly:

```bash
curl -fsSL -o ui/goose2/src/assets/home/world-cube.png  "<URL for imgImage139>"
curl -fsSL -o ui/goose2/src/assets/home/figure.png      "<URL for imgImage6090>"
curl -fsSL -o ui/goose2/src/assets/home/clock.svg       "<URL>"
curl -fsSL -o ui/goose2/src/assets/home/sticky-note.svg "<URL>"
```

Verify every file's extension matches its actual content:

```bash
file ui/goose2/src/assets/home/*
```

Expected: `.png` files report `PNG image data, <dims>`; `.svg` files report `SVG Scalable Vector Graphics image`. If anything is misreported, rename the file to match actual content (this is the asset-export fallback pattern; see spec §10.1).

- [ ] **Step 4: Identify and download Skills category assets**

From Skills — Full frame (`234:2240`), identify six category background photos + the mask. **Skip Research** — its Figma export on 2026-04-23 was a 5MB animated GIF, which triggers the asset-export fallback pattern (spec §10.1): Research renders a CSS gradient in Task 3.2 instead of an image file.

- `imgImage109` (Technical)
- `imgImage110` (Creative)
- `imgImage123` (Business)
- `imgImage130` (People)
- `imgImage131` (Personal Productivity)
- `imgImage113` (Finance)
- `imgImage108` (the 260×260 mask shape — exports as SVG, not PNG)

Download:

```bash
curl -fsSL -o ui/goose2/src/assets/skills/technical.png             "<URL for imgImage109>"
curl -fsSL -o ui/goose2/src/assets/skills/creative.png              "<URL for imgImage110>"
curl -fsSL -o ui/goose2/src/assets/skills/business.png              "<URL for imgImage123>"
curl -fsSL -o ui/goose2/src/assets/skills/people.png                "<URL for imgImage130>"
curl -fsSL -o ui/goose2/src/assets/skills/personal-productivity.png "<URL for imgImage131>"
curl -fsSL -o ui/goose2/src/assets/skills/finance.png               "<URL for imgImage113>"
curl -fsSL -o ui/goose2/src/assets/skills/tile-mask.svg             "<URL for imgImage108>"
```

Verify: `file ui/goose2/src/assets/skills/*` — `.png` files should report `PNG image data`; `tile-mask.svg` should report `SVG Scalable Vector Graphics image`. Rename to match actual content if anything mismatches.

- [ ] **Step 5: Identify and download the Agents figure asset**

From Agents — More than 5 frame (`114:1485`), there is a **single** figure asset (`imgImage6090`) rendered at 10 positions across 2 rows with different crops/positioning. The visual variety in Alex's mock is positional, not per-image. Confirmed on inspection 2026-04-23; single-source is intentional.

Download one file:

```bash
curl -fsSL -o ui/goose2/src/assets/agents/figure.png "<URL for imgImage6090>"
```

Verify with `file ui/goose2/src/assets/agents/figure.png`. All personas in Task 4.2 render this same image identically — uniformity signals the deferred-to-real-avatar-spec nature of this demo.

- [ ] **Step 6: Sanity check total disk usage**

```bash
du -sh ui/goose2/src/assets/*
```

Expected: each subfolder under ~2MB (individual PNGs typically 100-500KB).

- [ ] **Step 7: Do NOT commit yet**

---

### Task 1.3: Update `AppShell.tsx` root

**Files:**
- Modify: `ui/goose2/src/app/AppShell.tsx`

- [ ] **Step 1: Locate the return statement root div**

Search for `<div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">` (around line 465 of origin/main's AppShell.tsx).

- [ ] **Step 2: Swap tokens on root div**

Replace the `className` prop:

```tsx
// before
<div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">

// after
<div className="flex h-screen w-screen flex-col overflow-hidden bg-dot-grid text-[var(--text-default-alex)]">
```

- [ ] **Step 3: Add placeholder for GlobalComposerPill mount point**

Find the closing `</div>` of the AppShell root div (the one that holds TopBar + flex-row + StatusBar + SettingsModal + CreateProjectDialog). Immediately before it, add:

```tsx
{activeView !== "chat" && <GlobalComposerPill activeView={activeView} onSend={createNewTab} />}
```

- [ ] **Step 4: Add import for GlobalComposerPill at top of file**

```tsx
import { GlobalComposerPill } from "@/shared/ui/GlobalComposerPill";
```

**Note:** The file does not yet exist — that's created in Task 1.6. The import will not resolve until then, and typecheck will fail at this step. That's expected. Tasks 1.4 and 1.5 are parallel-safe text edits that don't run the app; leave the import in place and skip running dev server until Task 1.6 lands the component.

- [ ] **Step 5: Confirm the edit — read the `return` statement**

Run: `grep -n "bg-dot-grid\|GlobalComposerPill" ui/goose2/src/app/AppShell.tsx`

Expected: two lines matching — one for the className change and one for the conditional mount. If more than two, the changes are in the wrong place.

- [ ] **Step 6: Do NOT commit**

---

### Task 1.4: Rewrite `TopBar.tsx`

**Files:**
- Rewrite: `ui/goose2/src/app/ui/TopBar.tsx`

- [ ] **Step 1: Replace entire file contents**

```tsx
import { useTranslation } from "react-i18next";
import { cn } from "@/shared/lib/cn";
import { Button } from "@/shared/ui/button";
import type { AppView } from "@/app/AppShell";

interface TopBarProps {
  onSettingsClick?: () => void;
  activeView?: AppView;
  className?: string;
}

const PAGE_LABELS: Partial<Record<AppView, string>> = {
  skills: "Skills",
  agents: "Agents",
  projects: "Projects",
  "session-history": "Session History",
};

export function TopBar({ onSettingsClick, activeView, className }: TopBarProps) {
  const { t } = useTranslation("settings");
  const pageLabel = activeView ? PAGE_LABELS[activeView] : undefined;

  return (
    <header
      className={cn(
        "flex h-12 items-center gap-3 pl-20 pr-3",
        className,
      )}
      data-tauri-drag-region
    >
      <h1
        className="font-sans text-[24px] leading-[0.96] tracking-[-0.04em] text-[var(--text-title-alex)]"
        data-tauri-drag-region
      >
        {/* TODO: wire to active Project.title when Projects page ships */}
        Tulsi's World
        {pageLabel && (
          <>
            <span className="text-[var(--text-muted-alex)] opacity-60"> / </span>
            <span className="text-[var(--text-muted-alex)]">{pageLabel}</span>
          </>
        )}
      </h1>

      <div className="min-w-0 flex-1" data-tauri-drag-region />

      <Button
        type="button"
        variant="ghost"
        onClick={onSettingsClick}
        className="h-8 rounded-full bg-[var(--surface-button)] px-3 text-[14px] text-black/70 hover:bg-[var(--surface-button)]/80"
        title={t("title")}
      >
        Settings
      </Button>
    </header>
  );
}
```

- [ ] **Step 2: Update `AppShell.tsx` to pass `activeView` to TopBar**

Open `AppShell.tsx`. Find the `<TopBar onSettingsClick={() => openSettings()} />` line. Replace with:

```tsx
<TopBar onSettingsClick={() => openSettings()} activeView={activeView} />
```

- [ ] **Step 3: Confirm edits**

Run: `grep -n "activeView" ui/goose2/src/app/ui/TopBar.tsx ui/goose2/src/app/AppShell.tsx`

Expected: 4+ hits across both files.

- [ ] **Step 4: Do NOT commit**

---

### Task 1.5: Re-skin `Sidebar.tsx`

**Files:**
- Modify: `ui/goose2/src/features/sidebar/ui/Sidebar.tsx`

- [ ] **Step 1: Locate the inner container div**

Open the file. Find the line with `className="flex h-full flex-col overflow-hidden rounded-xl border border-border bg-background ..."` (around line 220 of origin/main).

- [ ] **Step 2: Swap tokens on the inner container**

Replace with:

```tsx
<div className="flex h-full flex-col overflow-hidden rounded-chrome bg-[var(--surface-chrome)] [--muted-foreground:var(--text-muted-alex)]">
```

Key changes:
- `border border-border` → removed
- `bg-background` → `bg-[var(--surface-chrome)]`
- `rounded-xl` → `rounded-chrome`
- The CSS variable override at the end re-points Ghost's `--muted-foreground` to our Alex muted color, so existing children that consume `text-muted-foreground` pick up the new value without per-component edits.

- [ ] **Step 3: Restyle the inline search input container**

Find the div wrapping the search input (around line 260, with `border border-border`). Replace:

```tsx
// before (the inner search div)
"gap-2 border border-border px-2.5 py-1.5 text-xs text-muted-foreground hover:text-foreground hover:bg-transparent",

// after
"gap-2 border-b border-[var(--surface-button)] px-2.5 py-1.5 text-[var(--text-body-alex)] text-muted-foreground hover:text-foreground hover:bg-transparent",
```

Swaps the full border for a subtle bottom rule and upgrades the text size to 14px.

- [ ] **Step 4: Adjust section label styling in `SidebarProjectsSection`**

Open `ui/goose2/src/features/sidebar/ui/SidebarProjectsSection.tsx`. Find any section label renders (typically "Projects" heading or similar). Apply:

```tsx
<p className="text-[10px] text-[var(--text-default-alex)] opacity-25">
  {label}
</p>
```

Above each label, add a 1px rule:

```tsx
<div className="mx-3 mb-1 h-px bg-[var(--color-gray-200)]" />
```

This is "re-skin in place" — if you find no existing labels to apply this to, skip without inventing new sections (approach A preserves current IA).

- [ ] **Step 5: Confirm**

Run: `grep -n "rounded-chrome\|surface-chrome" ui/goose2/src/features/sidebar/ui/Sidebar.tsx`

Expected: at least 2 hits.

- [ ] **Step 6: Do NOT commit**

---

### Task 1.6: Create `GlobalComposerPill` component

**Files:**
- Create: `ui/goose2/src/shared/ui/GlobalComposerPill.tsx`
- Test: `ui/goose2/src/shared/ui/__tests__/GlobalComposerPill.test.tsx` (minimal smoke test)

- [ ] **Step 1: Write the smoke test first**

Create `ui/goose2/src/shared/ui/__tests__/GlobalComposerPill.test.tsx`:

```tsx
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { GlobalComposerPill } from "../GlobalComposerPill";

describe("GlobalComposerPill", () => {
  it("renders the universal 'Start a conversation' placeholder", () => {
    render(<GlobalComposerPill onSend={vi.fn()} />);
    expect(
      screen.getByPlaceholderText(/start a conversation/i),
    ).toBeInTheDocument();
  });

  it("calls onSend with the typed text when send is clicked", () => {
    const onSend = vi.fn();
    render(<GlobalComposerPill onSend={onSend} />);

    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "hello" } });
    fireEvent.click(screen.getByRole("button", { name: /send/i }));

    expect(onSend).toHaveBeenCalledWith("hello");
  });

  it("does not send when input is empty", () => {
    const onSend = vi.fn();
    render(<GlobalComposerPill onSend={onSend} />);

    fireEvent.click(screen.getByRole("button", { name: /send/i }));

    expect(onSend).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Run the test and watch it fail**

```bash
cd ui/goose2 && pnpm test -- GlobalComposerPill --run 2>&1 | tail -20
```

Expected: FAIL — module `../GlobalComposerPill` not found.

- [ ] **Step 3: Implement the component**

Create `ui/goose2/src/shared/ui/GlobalComposerPill.tsx`:

```tsx
import { useState } from "react";
import { Mic, ArrowUp } from "lucide-react";

interface GlobalComposerPillProps {
  onSend: (text: string) => void;
}

const PLACEHOLDER = "Start a conversation";

export function GlobalComposerPill({ onSend }: GlobalComposerPillProps) {
  const [text, setText] = useState("");

  const handleSend = () => {
    const trimmed = text.trim();
    if (!trimmed) return;
    onSend(trimmed);
    setText("");
  };

  return (
    <div
      className="fixed bottom-6 right-6 z-40 flex h-[68px] w-[482px] max-w-[calc(100vw-48px)] items-center gap-3 rounded-composer bg-[var(--surface-composer)] pl-[30px] pr-4"
      role="region"
      aria-label="Quick compose"
    >
      <input
        type="text"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            handleSend();
          }
        }}
        placeholder={PLACEHOLDER}
        className="flex-1 appearance-none border-0 bg-transparent text-[16px] leading-[20px] text-black/70 outline-none placeholder:text-black/70 focus:outline-none focus:ring-0"
      />

      <button
        type="button"
        className="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--surface-button)]"
        aria-label="Voice dictation"
        title="Voice dictation (coming soon)"
      >
        <Mic className="size-4 text-black/70" />
      </button>

      <button
        type="button"
        onClick={handleSend}
        className="flex h-8 w-10 items-center justify-center rounded-full bg-[var(--surface-button)]"
        aria-label="Send"
      >
        <ArrowUp className="size-4 text-black/70" />
      </button>
    </div>
  );
}
```

Notes (per design review 2026-04-27):
- Single universal placeholder, not per-view — pill reads as a universal compose surface.
- `appearance-none border-0 outline-none focus:outline-none focus:ring-0` on the `<input>` kills macOS WebKit's native focus chrome (border + focus ring) that broke the translucent restraint.
- `activeView` prop dropped — no per-view branching needed. AppShell stops passing it.

- [ ] **Step 4: Run the test to verify it passes**

```bash
cd ui/goose2 && pnpm test -- GlobalComposerPill --run 2>&1 | tail -20
```

Expected: 3 tests PASS.

- [ ] **Step 5: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -10
```

Expected: baseline 4 errors only.

- [ ] **Step 6: Do NOT commit**

---

### Task 1.7: Auto-submit pill text via pending-first-message slice

**Files:**
- Modify: `ui/goose2/src/features/chat/stores/chatStore.ts` (slice extension)
- Modify: `ui/goose2/src/features/chat/hooks/useChatSessionController.ts` (consumer useEffect)
- Modify: `ui/goose2/src/app/AppShell.tsx` (handler swap)
- Add: unit test for the slice (next to existing chatStore tests)

Why: design review 2026-04-27. Earlier (2026-04-23) plan used `setDraft` to seed the new session's composer; the user then had to click send. Tulsi's review: the pill's value over "navigate to chat, then type" is one-shot send — the intermediate step reads as redundant. Switching to auto-submit on session-init.

**Architecture: pending-first-message slice on `chatStore`.**

The reason a slice is needed: `sendMessage` is exposed via the `useChat` hook, scoped to a `sessionId`. It can only be called from inside ChatView's render tree, not from `AppShell.handleGlobalCompose`. Pattern: AppShell stages a "pending first message" keyed by sessionId; `useChatSessionController` consumes it on session-init via useEffect and calls `sendMessage`.

- [ ] **Step 1: Verify chatStore + useChatSessionController shape (sanity check)**

```bash
grep -n "draftsBySession\|setDraft\|getSessionRuntime" ui/goose2/src/features/chat/stores/chatStore.ts | head -20
grep -n "useEffect\|sendMessage\b" ui/goose2/src/features/chat/hooks/useChatSessionController.ts | head -20
```

Expected: a `chatStore.ts` with Zustand-style state + actions; `useChatSessionController.ts` already has multiple `useEffect` blocks for session prep and exposes `sendMessage`.

- [ ] **Step 2: Add the slice to `chatStore.ts`**

In the state interface, add:

```ts
pendingFirstMessageBySession: Record<string, string>;
```

Initialize it to `{}` in the store creation.

In the actions interface (and implementation), add:

```ts
setPendingFirstMessage: (sessionId: string, text: string) => void;
consumePendingFirstMessage: (sessionId: string) => string | undefined;
```

Implementation:

```ts
setPendingFirstMessage: (sessionId, text) =>
  set((state) => ({
    pendingFirstMessageBySession: {
      ...state.pendingFirstMessageBySession,
      [sessionId]: text,
    },
  })),

consumePendingFirstMessage: (sessionId) => {
  const text = get().pendingFirstMessageBySession[sessionId];
  if (text === undefined) return undefined;
  set((state) => {
    const { [sessionId]: _, ...rest } = state.pendingFirstMessageBySession;
    return { pendingFirstMessageBySession: rest };
  });
  return text;
},
```

`consume` is one-shot: reads, deletes, returns. This guarantees we send exactly once even if the consuming useEffect runs multiple times.

- [ ] **Step 3: Add consumer useEffect in `useChatSessionController.ts`**

After the existing setup useEffects, add:

```ts
useEffect(() => {
  if (!sessionId) return;
  const pending = useChatStore.getState().consumePendingFirstMessage(sessionId);
  if (pending) {
    void sendMessage(pending);
  }
}, [sessionId, sendMessage]);
```

Notes:
- Use `useChatStore.getState()` (imperative, no subscription) — we don't want this useEffect to re-run when the pending map changes for some other reason. We only want it on session-init.
- `sendMessage` is captured stable via useCallback in `useChat`; deps are correct as-is.
- If `sendMessage` no-ops because `currentChatState` isn't `"idle"` (e.g., session not fully initialized yet), the pending message is already consumed and lost. Mitigation: place this useEffect AFTER any session-prep useEffects that initialize provider/agent state, so the controller is ready by the time we reach it. Verify by tracing: provider selection, workspace prep, then auto-send.

If during implementation the sendMessage call no-ops on a brand-new session (race condition), fall back to: keep the consume one-shot but defer the sendMessage call by one tick (e.g., via `queueMicrotask` or `setTimeout(..., 0)`), or guard on a "session ready" signal exposed by useChat. Adapt and report.

- [ ] **Step 4: Update `AppShell.handleGlobalCompose`**

Replace the existing `chatStore.setDraft(session.id, text)` call:

```tsx
const handleGlobalCompose = useCallback(
  async (text: string) => {
    const session = await createNewTab(DEFAULT_CHAT_TITLE);
    chatStore.setPendingFirstMessage(session.id, text);
  },
  [createNewTab, chatStore],
);
```

Same dep array; just swap the action name + drop the second arg shape.

- [ ] **Step 5: Add unit test for the slice**

Locate the existing chatStore test file (check `ui/goose2/src/features/chat/stores/__tests__/` or colocated `chatStore.test.ts`). Add a small describe block:

```ts
describe("pending first message", () => {
  it("setPendingFirstMessage stores by sessionId", () => {
    const store = useChatStore.getState();
    store.setPendingFirstMessage("s1", "hello");
    expect(useChatStore.getState().pendingFirstMessageBySession.s1).toBe("hello");
  });

  it("consumePendingFirstMessage returns and clears", () => {
    const store = useChatStore.getState();
    store.setPendingFirstMessage("s1", "hello");
    expect(store.consumePendingFirstMessage("s1")).toBe("hello");
    expect(store.consumePendingFirstMessage("s1")).toBeUndefined();
    expect(useChatStore.getState().pendingFirstMessageBySession.s1).toBeUndefined();
  });
});
```

Adapt store-access pattern to whatever the existing tests use (some tests grab actions via `useChatStore.getState()`, some via `renderHook`).

- [ ] **Step 6: Run typecheck + tests**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
cd ui/goose2 && pnpm test --run 2>&1 | tail -10
```

Expected: 0 typecheck errors. Tests pass (existing 474 + new slice tests).

- [ ] **Step 7: Do NOT commit**

---

### Task 1.8: Visual verification of Foundation phase

**Files:** none

- [ ] **Step 1: Start dev server**

```bash
cd ui/goose2 && pnpm tauri dev
```

- [ ] **Step 2: Compare to Alex's Landing (image 2) + Q2 2026 Figma**

Open the Tauri window. Verify each:

- Canvas color is `#dedede` (soft warm grey); dot grid at 24px visible at low alpha
- Sidebar has translucent white surface, rounded corners, no border, no shadow
- Topbar shows "Tulsi's World" at 24px Cash Sans (not Inter / system); breadcrumb appears when navigating to Skills/Agents/Session History
- Settings pill is text, not an icon, bg `#f5f5f5`
- Floating Global Composer Pill bottom-right on Home/Skills/Agents/Sessions views, hidden on Chat
- Typing in the pill + Enter creates a new chat (test on Home view)
- No shadows anywhere on re-skinned surfaces

- [ ] **Step 3: Test typecheck + lint + existing unit tests**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
cd ui/goose2 && pnpm lint 2>&1 | tail -5
cd ui/goose2 && pnpm test --run 2>&1 | tail -10
```

Expected: typecheck 4 baseline errors, lint clean, all existing tests pass + 3 new `GlobalComposerPill` tests pass.

- [ ] **Step 4: If anything is off, fix before commit**

Common issues:
- Cash Sans didn't load → check network tab in devtools; verify CDN URL resolves
- Canvas color wrong → check `bg-dot-grid` class is applied to root
- Sidebar looks the same → hard-refresh (Cmd+R in Tauri window)

---

### Task 1.9: Commit 1 — Foundation

**Files:** all of the above

- [ ] **Step 1: Stage all foundation work**

```bash
git add \
  ui/goose2/docs/superpowers/specs/2026-04-23-alex-redesign-design.md \
  ui/goose2/docs/superpowers/plans/2026-04-23-alex-redesign.md \
  ui/goose2/src/shared/styles/globals.css \
  ui/goose2/src/app/AppShell.tsx \
  ui/goose2/src/app/ui/TopBar.tsx \
  ui/goose2/src/features/sidebar/ui/Sidebar.tsx \
  ui/goose2/src/features/sidebar/ui/SidebarProjectsSection.tsx \
  ui/goose2/src/shared/ui/GlobalComposerPill.tsx \
  ui/goose2/src/shared/ui/__tests__/GlobalComposerPill.test.tsx \
  ui/goose2/src/assets/
```

- [ ] **Step 2: Verify staged changes look right**

```bash
git status --short
git diff --cached --stat
```

Expected: only the foundation files listed, with reasonable line counts (globals.css: ~+80, AppShell.tsx: ~+15, TopBar.tsx: ~-15 +50, Sidebar.tsx: ~+3 -3, GlobalComposerPill.tsx: ~+70, test: ~+30, assets: binary files).

- [ ] **Step 3: Commit**

```bash
git commit -m "$(cat <<'EOF'
feat(goose2): Alex redesign — foundation (tokens, shell, global composer)

Commit 1 of a 5-commit visual refactor matching Alex's Q2 2026 Figma.
See docs/superpowers/specs/2026-04-23-alex-redesign-design.md for full
design rationale and docs/superpowers/plans/2026-04-23-alex-redesign.md
for the execution sequence.

- Cash Sans (Regular 400 + Regular Italic 400) self-hosted from Block CDN
- Canvas + dot grid tokens (24px spacing, 1px dots)
- Surface tokens for sidebar (translucent), chrome buttons, tiles
- Text color tokens (#242424 default, #7f7f7f muted, #19191a title)
- Typography scale (10/14/16/24) with no bold
- Radii tokens (16 chrome, 40 composer, 20 tile)
- No-shadow rule on re-skinned surfaces
- color-scheme: light lock (dark mode deferred)
- AppShell: bg-dot-grid root, GlobalComposerPill mount (hidden on chat)
- TopBar: rewrite as breadcrumb title + Settings text pill
- Sidebar: in-place token re-skin (approach A, preserve IA)
- GlobalComposerPill: new floating pill, wires to createNewTab

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 4: Verify commit landed**

```bash
git log --oneline -1
git show --stat HEAD | head -30
```

Expected: one new commit showing the changed files.

---

## Phase 2 — Chat (Commit 2)

### Task 2.1: Wrap `ChatView` in bounded card

**Files:**
- Modify: `ui/goose2/src/features/chat/ui/ChatView.tsx`

- [ ] **Step 1: Locate the return statement**

Search for `<div className="relative flex h-full min-w-0">` (around line 76 of origin/main's ChatView.tsx).

- [ ] **Step 2: Replace the structure**

```tsx
// before
return (
  <ArtifactPolicyProvider ...>
    <div className="relative flex h-full min-w-0">
      <div className="flex min-w-0 flex-1 flex-col pr-1">
        {/* ...MessageTimeline, LoadingGoose, ChatInput... */}
      </div>
      {/* ...ChatContextPanel... */}
    </div>
  </ArtifactPolicyProvider>
);

// after
return (
  <ArtifactPolicyProvider ...>
    <div className="relative flex h-full min-w-0 p-4">
      <div className="flex min-w-0 flex-1 flex-col overflow-hidden rounded-card-chat bg-[var(--surface-card)]">
        {/* ...MessageTimeline, LoadingGoose, ChatInput... */}
      </div>
      {/* ...ChatContextPanel... */}
    </div>
  </ArtifactPolicyProvider>
);
```

Key changes:
- Outer flex: `+ p-4` (canvas padding around card)
- Inner column: `pr-1` dropped (canvas padding replaces it), `+ overflow-hidden rounded-card-chat bg-[var(--surface-card)]`

- [ ] **Step 3: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
```

Expected: baseline 4 errors.

- [ ] **Step 4: Do NOT commit**

---

### Task 2.2: Extract exact chat card radius from Figma

**Files:**
- Modify: `ui/goose2/src/shared/styles/globals.css`

- [ ] **Step 1: Pull the Chat frame via Figma MCP**

Use `mcp__figma__get_design_context` with `nodeId: "227:859"`, `fileKey: "e43a6gyBVn1SdARFkZpN0N"`, `excludeScreenshot: true`. In the generated code, find the outer "chat card" wrapper (the white rounded surface that contains the conversation + composer). Look for a class like `rounded-[20px]` or `rounded-[24px]` on a div with a white background.

- [ ] **Step 2: Update `--radius-card-chat` in `globals.css`**

Find the line `--radius-card-chat: 20px;` in `:root`. Replace `20px` with the extracted value. Keep at `20px` if that's what Figma shows (reasonable default).

- [ ] **Step 3: Dev server visual check**

```bash
cd ui/goose2 && pnpm tauri dev
```

Navigate to a chat. Card should look right — white surface, rounded at the extracted radius, canvas visible in the 16px gap around it.

- [ ] **Step 4: Do NOT commit**

---

### Task 2.3: Re-skin `ChatInput` outer container

**Files:**
- Modify: `ui/goose2/src/features/chat/ui/ChatInput.tsx`

- [ ] **Step 1: Locate the outermost container**

Open the file. The outermost `<div>` or `<form>` in the return statement will have the composer container styling. Scan for `border`, `shadow`, or background classes.

- [ ] **Step 2: Remove border, shadow, and solid background**

The composer now sits inside the white chat card, so its own surface should be transparent. Remove any `border`, `border-border`, `shadow-*`, `bg-background`, etc. from the outermost container. Replace with:

```tsx
<div className="flex flex-col border-t border-[var(--color-gray-200)] bg-transparent">
  {/* ...existing inner content... */}
</div>
```

Or whatever minimal border-top rule reads as "this is a distinct section of the card" without competing visually with the card surface. If that reads too heavy, drop the border-top entirely.

- [ ] **Step 3: Update placeholder to Alex's phrasing**

Locate the `<textarea>` or `<input>` with a `placeholder` prop. Replace with:

```tsx
placeholder="Type / for skill and @ to mention"
```

(If an i18n key is used, update the English translation file alongside.)

- [ ] **Step 4: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
```

Expected: baseline.

- [ ] **Step 5: Do NOT commit**

---

### Task 2.4: Create `HomeView` with static widgets

**Files:**
- Create: `ui/goose2/src/features/home/ui/HomeView.tsx`
- Modify: `ui/goose2/src/app/ui/AppShellContent.tsx`

- [ ] **Step 1: Create the directory**

```bash
mkdir -p ui/goose2/src/features/home/ui
```

- [ ] **Step 2: Implement HomeView**

Create `ui/goose2/src/features/home/ui/HomeView.tsx`:

```tsx
import worldCubeUrl from "@/assets/home/world-cube.png";
import clockUrl from "@/assets/home/clock.svg";
import figureUrl from "@/assets/home/figure.png";
import stickyNoteUrl from "@/assets/home/sticky-note.svg";

/**
 * Static placeholder composition evoking Alex's Landing frame.
 * Positions are percentage-based so the widgets reflow on viewport resize.
 * No interactivity — real widget canvas is a separate spec.
 */
export function HomeView() {
  return (
    <div className="relative h-full w-full overflow-hidden">
      {/* Blurry world cube, roughly centered */}
      <img
        src={worldCubeUrl}
        alt=""
        aria-hidden="true"
        className="absolute left-[35%] top-[20%] w-[40%] max-w-[700px] select-none"
      />

      {/* Clock top-right */}
      <img
        src={clockUrl}
        alt=""
        aria-hidden="true"
        className="absolute right-[5%] top-[5%] w-[12%] max-w-[200px] select-none"
      />

      {/* Three figures scattered — same asset reused at different positions/sizes,
          matching Alex's Landing frame (one figure PNG, positional variety only) */}
      <img
        src={figureUrl}
        alt=""
        aria-hidden="true"
        className="absolute left-[18%] top-[20%] w-[8%] max-w-[130px] select-none"
      />
      <img
        src={figureUrl}
        alt=""
        aria-hidden="true"
        className="absolute right-[8%] top-[35%] w-[6%] max-w-[100px] select-none"
      />
      <img
        src={figureUrl}
        alt=""
        aria-hidden="true"
        className="absolute left-[45%] top-[65%] w-[6%] max-w-[100px] select-none"
      />

      {/* Sticky note with red dot, bottom-left */}
      <img
        src={stickyNoteUrl}
        alt=""
        aria-hidden="true"
        className="absolute bottom-[15%] left-[15%] w-[18%] max-w-[300px] select-none"
      />
    </div>
  );
}
```

The specific left/top/width percentages are approximations from Alex's Landing frame — adjust after visual check. Images use `alt=""` + `aria-hidden="true"` because they're purely decorative.

- [ ] **Step 3: Route `activeView === "home"` to HomeView**

Open `ui/goose2/src/app/ui/AppShellContent.tsx`. Find the section that renders based on `activeView`. Add:

```tsx
import { HomeView } from "@/features/home/ui/HomeView";
```

```tsx
// In the render switch, add case:
if (activeView === "home") {
  return <HomeView />;
}
```

**Important:** The existing home-case may already render something (getting-started, etc.). Replace it — this is a visual placeholder for the exploration branch. The real home rendering lives in a separate future spec.

- [ ] **Step 4: Dev server visual check**

```bash
cd ui/goose2 && pnpm tauri dev
```

Navigate to Home. Should see the widget composition — cube, figures, clock, sticky note — floating on dot-grid canvas. Resize window; widgets reflow proportionally.

Tune the percentages in HomeView.tsx if anything overlaps or misaligns badly.

- [ ] **Step 5: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
```

Expected: baseline 4 errors.

- [ ] **Step 6: Do NOT commit**

---

### Task 2.5: Move ChatContextPanel inside the chat card

**Scope expansion (2026-04-27):** Visual review at this checkpoint
showed the panel rendering as a separate floating card beside the chat
card. Tulsi directed it to live inside the card — opening from the
right edge of the card itself. Spec §5.3 updated; plan Task 2.5 expands
from pure visual verification to a small structural edit.

**Files:**
- Modify: `ui/goose2/src/features/chat/ui/ChatView.tsx`
- Modify: `ui/goose2/src/features/chat/ui/ChatContextPanel.tsx`

- [ ] **Step 1: Restructure ChatView's chat card to flex-row**

Old:

```tsx
<div className="flex min-w-0 flex-1 flex-col overflow-hidden rounded-card-chat bg-[var(--surface-card)]">
  {/* MessageTimeline + LoadingGoose + ChatInput */}
</div>

<ChatContextPanel ... />
```

New (drop `flex-col` from card; wrap conversation in inner column;
move `<ChatContextPanel>` inside the card):

```tsx
<div className="flex min-w-0 flex-1 overflow-hidden rounded-card-chat bg-[var(--surface-card)]">
  <div className="flex min-w-0 flex-1 flex-col">
    {/* MessageTimeline + LoadingGoose + ChatInput */}
  </div>
  <ChatContextPanel ... />
</div>
```

- [ ] **Step 2: Strip the panel's own surface**

In `ChatContextPanel.tsx`, locate the inner `<aside>`:

Old: `<aside className="flex min-w-0 flex-1 overflow-hidden rounded-xl border border-border bg-background">`

New: `<aside className="flex min-w-0 flex-1 overflow-hidden border-l border-[var(--color-gray-200)]">`

The panel now sits flush inside the white chat card with a subtle left
divider. Internal sub-cards (Workspace / Changes / Extensions) keep
their existing styling.

- [ ] **Step 3: Verify width animation + toggle position**

Start the dev server and toggle the panel:
- Panel closed: chat card occupies the full available width
- Panel opening: panel slides in from right edge inside the card over
  ~200ms; conversation column flex-shrinks to make room
- Panel closing: conversation re-expands smoothly
- Toggle button continues to land at the top-right corner of the chat
  card across both states

If the toggle drifts or the animation judders, the absolute positioning
in ChatContextPanel.tsx (lines ~75-96) targets ChatView's outer
`relative` div as its containing block — that didn't change, so this
should still work.

- [ ] **Step 4: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
```

Expected: 0 errors.

- [ ] **Step 5: Do NOT commit**

---

### Task 2.6: Commit 2 — Chat

- [ ] **Step 1: Stage Chat phase changes**

```bash
git add \
  ui/goose2/src/features/chat/ui/ChatView.tsx \
  ui/goose2/src/features/chat/ui/ChatInput.tsx \
  ui/goose2/src/features/home/ \
  ui/goose2/src/app/ui/AppShellContent.tsx \
  ui/goose2/src/shared/styles/globals.css
```

- [ ] **Step 2: Verify diff**

```bash
git diff --cached --stat
```

- [ ] **Step 3: Commit**

```bash
git commit -m "$(cat <<'EOF'
feat(goose2): Alex redesign — chat bounded card + home widget placeholders

Commit 2 of 5. See spec at docs/superpowers/specs/2026-04-23-alex-redesign-design.md.

- ChatView wraps conversation+composer in a bounded white card on canvas
- ChatInput outer container surface-less (sits on card's own surface)
- Home view renders static decorative widget placeholders (world cube,
  clock, three figures, sticky note) — no drag/persistence, pure visual
- Chat card radius finalized from Figma frame 227:859
- Context panel flex-resize behavior verified; no structural changes

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Phase 3 — Skills (Commit 3)

### Task 3.1: Create `CategoryHeroTile` component

**Files:**
- Create: `ui/goose2/src/features/skills/ui/CategoryHeroTile.tsx`

- [ ] **Step 1: Implement**

```tsx
interface CategoryHeroTileProps {
  label: string;
  /** Masked photo background. Provide EITHER `imageUrl` (photo) OR `gradient` (CSS gradient fallback). If both are provided, `imageUrl` wins. */
  imageUrl?: string;
  /** CSS gradient string used when Figma's export for this tile is unusable (see spec §10.1 asset-export fallback pattern). Example: `"linear-gradient(135deg, #6b8f5a, #3e5a3e)"`. */
  gradient?: string;
  /** Rounded mask shape applied over the background. */
  maskUrl: string;
}

/**
 * Static decorative category tile for the Skills page top row.
 * Non-functional; categories are not a real feature yet — visual
 * placeholder pending real category feature (separate spec).
 */
export function CategoryHeroTile({
  label,
  imageUrl,
  gradient,
  maskUrl,
}: CategoryHeroTileProps) {
  const backgroundStyle = imageUrl
    ? {
        backgroundImage: `url(${imageUrl})`,
        backgroundSize: "cover",
        backgroundPosition: "center",
      }
    : {
        backgroundImage: gradient,
      };

  return (
    <div className="relative h-[260px] w-[260px] overflow-hidden rounded-tile">
      {/* Masked background (photo or gradient fallback) */}
      <div
        className="absolute inset-0"
        style={{
          maskImage: `url(${maskUrl})`,
          WebkitMaskImage: `url(${maskUrl})`,
          maskSize: "cover",
          WebkitMaskSize: "cover",
          filter: imageUrl ? "blur(2px)" : undefined,
          ...backgroundStyle,
        }}
      />
      {/* Slight dark overlay for label contrast */}
      <div className="absolute inset-0 bg-black/20" />
      {/* Label top-left, whitespace allowed to wrap */}
      <p className="absolute left-5 top-5 max-w-[200px] text-[24px] leading-[0.96] tracking-[-0.02em] text-white">
        {label}
      </p>
    </div>
  );
}
```

Why two fill modes: six of the seven Skills categories have clean photo exports from Figma. Research's export was a 5MB animated GIF (see spec §10.1 asset-export fallback pattern). Rather than ship the GIF or chase a cleaner Figma export, Research uses a CSS gradient matching the earth-toned vibe of Alex's tile. One tile differing from the rest reads as a design variant, not an oversight.

- [ ] **Step 2: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
```

Expected: baseline.

- [ ] **Step 3: Do NOT commit**

---

### Task 3.2: Render category hero tile row in `SkillsView`

**Files:**
- Modify: `ui/goose2/src/features/skills/ui/SkillsView.tsx`

- [ ] **Step 1: Add asset imports at top of file**

```tsx
import technicalUrl from "@/assets/skills/technical.png";
import creativeUrl from "@/assets/skills/creative.png";
import businessUrl from "@/assets/skills/business.png";
import peopleUrl from "@/assets/skills/people.png";
import personalProductivityUrl from "@/assets/skills/personal-productivity.png";
import financeUrl from "@/assets/skills/finance.png";
import tileMaskUrl from "@/assets/skills/tile-mask.svg";
import { CategoryHeroTile } from "./CategoryHeroTile";
```

Note: no `researchUrl` import. Research's tile renders a CSS gradient — see spec §10.1 asset-export fallback pattern. `tile-mask` is an SVG (Figma exported the mask shape as SVG, not PNG).

- [ ] **Step 2: Define category config above the component**

```tsx
type CategoryTile =
  | { label: string; imageUrl: string }
  | { label: string; gradient: string };

const CATEGORY_TILES: Array<CategoryTile> = [
  { label: "Technical\nSkills", imageUrl: technicalUrl },
  { label: "Creative\nSkills", imageUrl: creativeUrl },
  { label: "Business\nSkills", imageUrl: businessUrl },
  { label: "People\nSkills", imageUrl: peopleUrl },
  { label: "Personal\nProductivity\nSkills", imageUrl: personalProductivityUrl },
  { label: "Research\nSkills", gradient: "linear-gradient(135deg, #6b8f5a 0%, #3e5a3e 100%)" },
  { label: "Finance\nSkills", imageUrl: financeUrl },
];
```

Tune the green stops on the Research gradient after visual check in Task 3.7 if they clash with Alex's earth-tone palette in adjacent tiles.

- [ ] **Step 3: Render the category row at the top of the view**

Inside the `SkillsView` return, above the skill-cards grid, add a category hero row. Grid: 4 columns × 2 rows = 8 slots (7 categories + 1 empty-state slot handled separately).

```tsx
<section className="mb-10 grid grid-cols-4 gap-8">
  {CATEGORY_TILES.map((cat) => (
    <CategoryHeroTile
      key={cat.label}
      label={cat.label.replace(/\n/g, " ")}
      imageUrl={"imageUrl" in cat ? cat.imageUrl : undefined}
      gradient={"gradient" in cat ? cat.gradient : undefined}
      maskUrl={tileMaskUrl}
    />
  ))}
</section>
```

The `\n` in the label is replaced with space for now; multi-line labels can come later if the tile design wants them (Alex's tiles show "People / Skills" on two lines — we can add `whitespace-pre-line` if you want the literal breaks).

- [ ] **Step 4: Visual check**

```bash
cd ui/goose2 && pnpm tauri dev
```

Navigate to Skills. Should see 7 category tiles in a row-grid above the existing skill cards.

- [ ] **Step 5: Do NOT commit**

---

### Task 3.3: Rework individual skill card

**Files:**
- Modify: `ui/goose2/src/features/skills/ui/SkillsView.tsx`

- [ ] **Step 1: Define the tag pill color cycle helper**

Near the top of the file (after imports, before the component):

```tsx
const TAG_PILL_COLORS = [
  "var(--pill-pink)",
  "var(--pill-olive)",
  "var(--pill-blue)",
] as const;

function tagPillColor(index: number): string {
  return TAG_PILL_COLORS[index % TAG_PILL_COLORS.length];
}
```

- [ ] **Step 2: Replace the skill card render in `SkillsView`**

Locate the existing skill card render (likely a `.map()` over `skills`). Replace each card's JSX with:

```tsx
{skills.map((skill, index) => (
  <div
    key={skill.name}
    className="group relative h-[260px] w-[260px] overflow-hidden rounded-tile bg-[var(--surface-tile)] p-5"
  >
    {/* Tag pill (skill name, colored) */}
    <span
      className="inline-flex h-5 items-center rounded-full px-[6px] pb-[3px] text-[14px] text-[var(--text-title-alex)]"
      style={{ backgroundColor: tagPillColor(index) }}
    >
      {skill.name}
    </span>

    {/* Description */}
    <p className="mt-8 line-clamp-7 text-[16px] leading-[20px] text-[var(--text-muted-alex)]">
      {skill.description}
    </p>

    {/* Install button, hover-only — DECORATIVE PLACEHOLDER.
        Current goose2 skills are user-created, not installed from a catalog,
        so there is no real install flow to wire. This button is visual-only
        for this branch; a real install affordance is a separate spec. */}
    <button
      type="button"
      className="absolute bottom-4 right-4 hidden h-8 rounded-full bg-[var(--surface-install)] px-3 text-[14px] text-black/70 group-hover:inline-flex group-hover:items-center"
      aria-label={`Install ${skill.name} (placeholder)`}
      tabIndex={-1}
    >
      Install
    </button>

    {/* Menu (hover) — preserve existing SkillCardMenu but style trigger */}
    <div className="absolute top-4 right-4 hidden group-hover:block">
      <SkillCardMenu
        skill={skill}
        onEdit={handleEdit}
        onDuplicate={handleDuplicate}
        onExport={handleExport}
        onDelete={handleDelete}
      />
    </div>
  </div>
))}
```

**Note:** The exact prop/handler names for edit/duplicate/etc. must match the existing `SkillCardMenu` contract at the top of `SkillsView.tsx`. Grep the file to confirm before editing:

```bash
grep -n "SkillCardMenu\|onEdit\|onDuplicate\|onExport\|onDelete" ui/goose2/src/features/skills/ui/SkillsView.tsx | head -20
```

- [ ] **Step 3: Update the grid container to fit 260px cards**

Find the grid wrapping the `.map()`. Adjust:

```tsx
<section className="grid grid-cols-[repeat(auto-fill,260px)] gap-8">
  {/* ...skill card map... */}
</section>
```

- [ ] **Step 4: Line-clamp utility**

If `line-clamp-7` isn't already available, check if `@tailwindcss/line-clamp` is loaded. If not, add raw style:

```tsx
<p
  className="mt-8 text-[16px] leading-[20px] text-[var(--text-muted-alex)]"
  style={{
    display: "-webkit-box",
    WebkitBoxOrient: "vertical",
    WebkitLineClamp: 7,
    overflow: "hidden",
  }}
>
  {skill.description}
</p>
```

- [ ] **Step 5: Visual check**

Dev server → Skills page. Verify cards are 260x260, `#f5f5f5`, rounded-20, tag pills cycle through pink/olive/blue, descriptions 16px muted, Install button appears on hover.

- [ ] **Step 6: Do NOT commit**

---

### Task 3.4: Add new-skill empty state card

**Files:**
- Modify: `ui/goose2/src/features/skills/ui/SkillsView.tsx`

- [ ] **Step 1: Add the empty state card as the first child of the skills grid**

Place before the `.map()` (so it renders first in reading order):

```tsx
<section className="grid grid-cols-[repeat(auto-fill,260px)] gap-8">
  {/* New-skill empty state */}
  <div className="group relative h-[260px] w-[260px] overflow-hidden rounded-tile bg-[var(--surface-tile)] p-5">
    <span className="inline-flex h-5 items-center rounded-full bg-[var(--pill-neutral)] px-[6px] pb-[3px] text-[14px] text-[var(--text-title-alex)]">
      new-skill
    </span>

    <p className="mt-8 text-[16px] leading-[20px] text-[var(--text-muted-alex)]">
      Describe your new skill, e.g.{" "}
      <em className="italic">
        "Summarize any webpage into 3 bullet points when given a URL."
      </em>
    </p>

    <button
      type="button"
      onClick={() => setDialogOpen(true)}
      className="absolute bottom-4 right-4 flex h-8 w-10 items-center justify-center rounded-full bg-[var(--surface-install)]"
      aria-label="Create new skill"
    >
      <ArrowRight className="size-4 text-black/70" />
    </button>
  </div>

  {skills.map((skill, index) => (
    /* ...existing card from Task 3.3... */
  ))}
</section>
```

- [ ] **Step 2: Add `ArrowRight` to imports**

```tsx
import { /* ...existing..., */ ArrowRight } from "lucide-react";
```

- [ ] **Step 3: Visual check**

Empty state should be the first card. Italic placeholder visible. Arrow button opens the existing create-skill dialog.

- [ ] **Step 4: Do NOT commit**

---

### Task 3.5: Polish the inline toolbar above the grid

**Files:**
- Modify: `ui/goose2/src/features/skills/ui/SkillsView.tsx`

- [ ] **Step 1: Locate existing toolbar**

Find the existing `SearchBar`, "New Skill", "Import" buttons (likely in a flex row above the skills grid — search the file for `SearchBar` or `importInputRef`).

- [ ] **Step 2: Restyle to Alex's pill-toolbar**

Ensure each button renders as:

```tsx
<Button
  type="button"
  variant="ghost"
  className="h-8 rounded-full bg-[var(--surface-button)] px-3 text-[14px] text-black/70 hover:bg-[var(--surface-button)]/80"
>
  <Upload className="mr-2 size-4" />
  Import
</Button>
```

Apply equivalent styling to "Add New" / "List view" / "Sort" / "Search" buttons if present. Icon-only buttons use `h-8 w-[38px] p-0`.

The SearchBar component (`@/shared/ui/SearchBar`) should be re-skinned globally — but for now, ensure its outer wrapper gets the same bg-button + rounded-full treatment so it matches.

- [ ] **Step 3: Visual check**

Toolbar above grid should show matching pill buttons.

- [ ] **Step 4: Do NOT commit**

---

### Task 3.6: Add bottom fade gradient

**Files:**
- Modify: `ui/goose2/src/features/skills/ui/SkillsView.tsx`

- [ ] **Step 1: Add fade div at the bottom of the view**

Inside the top-level container of `SkillsView`, below the grid:

```tsx
<div
  className="pointer-events-none sticky bottom-0 left-0 h-48 w-full"
  style={{
    background: "linear-gradient(to bottom, rgba(222,222,222,0) 0%, var(--canvas) 100%)",
    backdropFilter: "blur(1.5px)",
    WebkitBackdropFilter: "blur(1.5px)",
  }}
  aria-hidden="true"
/>
```

`pointer-events-none` so it doesn't block clicks on cards below. `sticky bottom-0` so it rides the scroll position. Alternative: position it `fixed` relative to the main scroll container.

- [ ] **Step 2: Visual check**

Scroll down through Skills. Bottom rows should fade into canvas near the floating composer pill.

- [ ] **Step 3: Do NOT commit**

---

### Task 3.7: Visual verification of Skills phase

- [ ] **Step 1: Dev server check**

Navigate to Skills. Verify:
- 7 category hero tiles in top row (+ empty new-skill tile as first card in the grid below)
- Individual skill cards: 260x260, #f5f5f5, rounded-20, tag pills cycle 3 colors
- Install button on hover only
- Toolbar: pill-styled buttons
- Bottom fade gradient present

- [ ] **Step 2: Typecheck + lint**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
cd ui/goose2 && pnpm lint 2>&1 | tail -5
```

Expected: baseline.

---

### Task 3.8: Commit 3 — Skills

- [ ] **Step 1: Stage Skills phase changes**

```bash
git add \
  ui/goose2/src/features/skills/ui/SkillsView.tsx \
  ui/goose2/src/features/skills/ui/CategoryHeroTile.tsx
```

- [ ] **Step 2: Commit**

```bash
git commit -m "$(cat <<'EOF'
feat(goose2): Alex redesign — skills page (category tiles, cards, toolbar)

Commit 3 of 5. See spec at docs/superpowers/specs/2026-04-23-alex-redesign-design.md.

- 7 static decorative CategoryHeroTile tiles at top (placeholder for
  future category feature)
- Individual skill cards: 260x260 #f5f5f5 rounded-20 with skill name
  as tag pill cycling pink/olive/blue by index
- New-skill empty state card (italic placeholder, arrow submit)
- Inline toolbar pills (Import, Add New, List view, Sort, Search)
- Hover-only Install button (#dedede)
- Bottom fade gradient into canvas for polish

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Phase 4 — Agents (Commit 4)

### Task 4.1: ~~Write figure-assignment utility~~ REMOVED

Removed on 2026-04-23 after Figma inspection (Task 1.2): Alex's Agents frame uses a single figure asset re-rendered at different positions, not five distinct figures. With only one figure, there is nothing to hash/assign. All personas render the same image — uniformity signals the deferred real-avatar spec honestly.

Task 4.2 imports `src/assets/agents/figure.png` directly.

---

### Task 4.2: Replace `PersonaCard` body render

**Files:**
- Modify: `ui/goose2/src/features/agents/ui/PersonaCard.tsx`

- [ ] **Step 1: Add figure asset import at top**

```tsx
import figureUrl from "@/assets/agents/figure.png";
```

Single import — same figure renders for every persona per Task 4.1's removal rationale.

- [ ] **Step 2: Replace the return statement body**

```tsx
return (
  <div
    aria-label={t("card.ariaLabel", { name: persona.displayName })}
    role="button"
    onClick={() => !menuOpen && onSelect?.(persona)}
    onKeyDown={handleCardKeyDown}
    tabIndex={0}
    className={cn(
      "group relative flex flex-col items-center cursor-pointer px-3 py-4",
      "transition-colors duration-200",
      isActive && "bg-black/[0.03]",
    )}
  >
    {/* Figure cutout — same asset for every persona (decorative placeholder) */}
    <img
      src={figureUrl}
      alt=""
      aria-hidden="true"
      className="h-[220px] w-auto select-none"
    />

    {/* Thin horizontal rule */}
    <div className="mt-3 h-px w-[149px] bg-[var(--color-gray-200)]" />

    {/* Name pill */}
    <span className="mt-3 inline-flex h-5 items-center rounded-full bg-[var(--surface-button)] px-[6px] pb-[3px] text-[14px] text-[var(--text-title-alex)]">
      {persona.displayName}
    </span>

    {/* Description */}
    <p className="mt-3 w-[149px] text-[16px] leading-[20px] text-[var(--text-muted-alex)]">
      {persona.description}
    </p>

    {/* Menu (hover only) */}
    <div className="absolute top-2 right-2 hidden group-hover:block">
      <DropdownMenu open={menuOpen} onOpenChange={setMenuOpen}>
        <DropdownMenuTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            size="icon-xs"
            aria-label={t("card.optionsAria", { name: persona.displayName })}
            className="size-6 rounded-md text-muted-foreground hover:text-foreground"
          >
            <MoreVertical className="size-3.5" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" sideOffset={4}>
          {canEditPersona && (
            <DropdownMenuItem onSelect={() => onEdit?.(persona)}>
              <Pencil className="size-3.5" />
              {t("common:actions.edit")}
            </DropdownMenuItem>
          )}
          <DropdownMenuItem onSelect={() => onDuplicate?.(persona)}>
            <Copy className="size-3.5" />
            {t("common:actions.duplicate")}
          </DropdownMenuItem>
          {onExport && (
            <DropdownMenuItem onSelect={() => onExport(persona)}>
              <Download className="size-3.5" />
              {t("common:actions.export")}
            </DropdownMenuItem>
          )}
          {canDeletePersona && (
            <DropdownMenuItem
              variant="destructive"
              onSelect={() => onDelete?.(persona)}
            >
              <Trash2 className="size-3.5" />
              {t("common:actions.delete")}
            </DropdownMenuItem>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  </div>
);
```

Removes:
- The surrounding `rounded-xl border p-5 bg-background` classes
- The `<Avatar>` with circle crop
- The inline provider/model label (no dedicated slot in Alex's design — moves to persona detail view if still needed)
- The `ring-1 ring-ring` active state

Preserves:
- All handlers and functionality (select, edit, duplicate, delete, export)
- All the dropdown menu items
- `tabIndex={0}` + keyboard accessibility

**Note:** Grep for `canEditPersona` / `canDeletePersona` / `getPersonaSource` to confirm they're defined above — they are in the original file. Don't remove them.

- [ ] **Step 3: Remove unused imports**

Remove `Avatar`, `AvatarImage`, `AvatarFallback`, `Badge`, `useAvatarSrc`, `getPersonaSource` (if unused after changes). Let the typecheck tell you.

- [ ] **Step 4: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -10
```

Expected: baseline 4 errors only.

- [ ] **Step 5: Do NOT commit**

---

### Task 4.3: Adjust `PersonaGallery` grid

**Files:**
- Modify: `ui/goose2/src/features/agents/ui/PersonaGallery.tsx`

- [ ] **Step 1: Find the grid container**

Search for the `.map()` over `personas` and its wrapping element. It probably uses a responsive grid class like `grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4`.

- [ ] **Step 2: Adjust to a responsive 5-column max with auto-fill**

```tsx
<section className="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-x-4 gap-y-8 p-8">
  {personas.map((persona) => (
    <PersonaCard
      key={persona.id}
      persona={persona}
      activePersonaId={activePersonaId}
      /* ...existing props... */
    />
  ))}
</section>
```

`auto-fill, minmax(180px, 1fr)` gives us 2-5 columns depending on viewport width, with each cell at least 180px wide. Matches Alex's "scales fluidly" intent without conditionally branching on persona count.

- [ ] **Step 3: Remove any `<5` vs `>=5` conditional rendering branch**

If the existing `PersonaGallery` has a `personas.length < 5` branch that renders differently, delete it — one grid layout for all counts.

- [ ] **Step 4: Update `SkeletonCard` to match**

The skeleton card (top of the file) should roughly match the new card's proportions. Simplify:

```tsx
function SkeletonCard() {
  return (
    <div aria-hidden="true" className="flex flex-col items-center px-3 py-4">
      <Skeleton className="h-[220px] w-[110px]" />
      <Skeleton className="mt-3 h-px w-[149px]" />
      <Skeleton className="mt-3 h-5 w-20" />
      <Skeleton className="mt-3 h-4 w-full" />
      <Skeleton className="mt-1 h-4 w-3/4" />
    </div>
  );
}
```

- [ ] **Step 5: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
```

Expected: baseline.

- [ ] **Step 6: Do NOT commit**

---

### Task 4.4: Visual verification of Agents phase

- [ ] **Step 1: Dev server check**

Navigate to Agents. Verify:
- Figures displayed as tall cutouts (no circle crop)
- Horizontal rule line below each figure
- Name pill (#f5f5f5) centered below the rule
- Description (16px muted) below the pill, 149px wide
- Grid responsive — 2-5 columns depending on window width
- No row-of-4 vs grid branching (always grid)
- Hover shows `MoreHorizontal` menu trigger
- Clicking a persona still selects it (functionality intact)

- [ ] **Step 2: Run all tests**

```bash
cd ui/goose2 && pnpm test --run 2>&1 | tail -15
```

Expected: existing tests pass. No new tests added in this phase (Task 4.1 was removed).

---

### Task 4.5: Commit 4 — Agents

- [ ] **Step 1: Stage Agents phase**

```bash
git add \
  ui/goose2/src/features/agents/ui/PersonaCard.tsx \
  ui/goose2/src/features/agents/ui/PersonaGallery.tsx
```

- [ ] **Step 2: Commit**

```bash
git commit -m "$(cat <<'EOF'
feat(goose2): Alex redesign — agents cutout-figure cards

Commit 4 of 5. See spec at docs/superpowers/specs/2026-04-23-alex-redesign-design.md.

- PersonaCard body replaced: cutout figure + 1px rule + name pill + desc
- No surface, no border, no shadow on card
- Active state: subtle bg tint (replaces ring-1 ring-ring)
- Menu trigger appears on hover only
- PersonaGallery grid: responsive auto-fill 180px minmax
- Removed <5 vs >=5 conditional layout — one grid for all counts
- Single decorative figure rendered identically for every persona
  (Figma uses one figure asset reused positionally; uniformity signals
  the deferred real-avatar spec honestly)
- User-uploaded avatars ignored in this visual (demo-stage;
  real tall-figure avatar integration is a separate spec)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Phase 5 — Sessions (Commit 5)

### Task 5.1: Re-skin `SessionCard`

**Files:**
- Modify: `ui/goose2/src/features/sessions/ui/SessionCard.tsx`

- [ ] **Step 1: Locate the outer card div in the return statement**

Find the outermost `<div>` / `<button>` that wraps the card content. It probably has classes like `rounded border p-4 bg-background` or similar.

- [ ] **Step 2: Strip card surface**

Replace the outer container className with:

```tsx
<div
  className="group relative flex flex-col gap-1 rounded-md px-3 py-2 hover:bg-black/[0.02] cursor-pointer"
  /* ...existing handlers... */
>
  {/* ...existing inner content... */}
</div>
```

Removes: border, solid background, shadow. Keeps: hover tint, rounded interactive hit area, padding.

- [ ] **Step 3: Update text sizing and colors**

Inside the card, find:
- The title render → `className="text-[14px] text-[var(--text-default-alex)]"`
- The metadata (date, persona, project, updatedAt) row → `className="text-[10px] text-[var(--text-muted-alex)]"`
- The snippet (if present) → `className="text-[14px] text-[var(--text-muted-alex)] line-clamp-2"`

- [ ] **Step 4: Menu trigger on hover**

Wrap the `DropdownMenu` trigger in a container that's `hidden` by default and `group-hover:block`:

```tsx
<div className="absolute top-2 right-2 hidden group-hover:block">
  <DropdownMenu>...</DropdownMenu>
</div>
```

Add `relative` to the outer card div if it's not already there.

- [ ] **Step 5: Typecheck**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
```

Expected: baseline.

- [ ] **Step 6: Do NOT commit**

---

### Task 5.2: Polish `SessionHistoryView`

**Files:**
- Modify: `ui/goose2/src/features/sessions/ui/SessionHistoryView.tsx`

- [ ] **Step 1: Update page container padding**

Find the top-level return element. Apply:

```tsx
<div className="h-full overflow-y-auto p-8">
  {/* ...existing contents... */}
</div>
```

- [ ] **Step 2: Style grouped date headers**

Find where sections are rendered per date group (likely a `.map()` over `groupSessionsByDate` result). Render each section header as:

```tsx
<div className="mt-8 first:mt-0">
  <div className="mb-2 h-px w-full bg-[var(--color-gray-200)]" />
  <h2 className="mb-4 text-[10px] text-[var(--text-default-alex)] opacity-25">
    {toTitleCase(groupLabel)}
  </h2>
  {/* ...sessions in group, each as a SessionCard... */}
</div>
```

Where `toTitleCase` capitalizes labels like "today" → "Today" (or just pass the label through if it's already title case from `groupSessionsByDate`).

- [ ] **Step 3: Re-skin `SearchBar`**

The shared `SearchBar` component is imported — its re-styling is a global change. For now, wrap it to match this page:

```tsx
<div className="mb-6 max-w-xl">
  <SearchBar {/* ...existing props... */} />
</div>
```

If the `SearchBar` internals themselves still look boxy, that's fine for this pass — shared-component re-skin can land in a follow-up.

- [ ] **Step 4: Visual check**

Navigate to Session History. Grouped headers with divider above and opacity-25 label text. Page padding feels spacious.

- [ ] **Step 5: Do NOT commit**

---

### Task 5.3: Visual verification of Sessions phase

- [ ] **Step 1: Dev server check**

Session History:
- Canvas + dot grid visible
- Grouped date headers with divider-above + 10px opacity-25 label
- `SessionCard`s: no surface, hover tint, 14px title, 10px metadata, menu on hover
- Functionality intact (select, rename, archive, duplicate, export)

Sidebar Recents:
- Per Task 1.5, section labels have divider-above + title-case styling where present
- No IA change (approach A)

- [ ] **Step 2: Typecheck + lint + tests**

```bash
cd ui/goose2 && pnpm typecheck 2>&1 | tail -5
cd ui/goose2 && pnpm lint 2>&1 | tail -5
cd ui/goose2 && pnpm test --run 2>&1 | tail -15
```

Expected: 0 typecheck errors (new baseline as of 2026-04-23 — the 4 pre-existing SDK-drift errors the spec describes were fixed on `origin/main` before this branch was cut), lint clean, all existing tests + our new `GlobalComposerPill` test pass.

---

### Task 5.4: Commit 5 — Sessions

- [ ] **Step 1: Stage Sessions phase**

```bash
git add \
  ui/goose2/src/features/sessions/ui/SessionCard.tsx \
  ui/goose2/src/features/sessions/ui/SessionHistoryView.tsx
```

- [ ] **Step 2: Commit**

```bash
git commit -m "$(cat <<'EOF'
feat(goose2): Alex redesign — sessions surface-less cards + page polish

Commit 5 of 5. See spec at docs/superpowers/specs/2026-04-23-alex-redesign-design.md.

- SessionCard: no surface, hover tint, 14px title, 10px metadata
- Hover-only menu trigger (consistent with Skills/Agents)
- SessionHistoryView: p-8 page padding, grouped date headers with
  divider-above + 10px opacity-25 label (matches sidebar Pinned/Recents
  pattern from foundation)
- Functionality fully preserved (select, rename, archive, duplicate,
  export, search, grouping)

Closes the 5-commit Alex redesign sequence. See commits 1-5 for the
full arc. Dark mode, widget canvas, real categories, tall-figure avatar
integration, Projects page, Search page, Loader, and Context panel
styling all remain as separate specs.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 3: Verify history**

```bash
git log --oneline origin/main..HEAD
```

Expected: 5 commits — foundation, chat, skills, agents, sessions.

---

## Post-implementation checklist

- [ ] All 5 commits on `tulsi/visual-design`
- [ ] Typecheck: 0 errors (matches origin/main baseline as of 2026-04-23)
- [ ] Lint: clean (or baseline)
- [ ] All tests pass (existing + `GlobalComposerPill`)
- [ ] Dev server: each in-scope page visually matches Alex's Figma reference
- [ ] Spec + plan committed alongside foundation (commit 1)
- [ ] Branch ready to share / present
- [ ] Stash @{0} (abandoned exploration) decision: keep, drop (`git stash drop 0`), or archive on the dead `tulsi/visual-exploration` branch

---

*End of plan.*
