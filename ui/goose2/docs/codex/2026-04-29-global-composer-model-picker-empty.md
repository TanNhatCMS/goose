# Global composer model picker shows "No models available"

**Status:** open — needs technical investigation
**Branch:** `tulsi/visual-design`
**Affected file:** `src/shared/ui/GlobalComposerPill.tsx`
**Reference (working) file:** `src/features/chat/ui/AgentModelPicker.tsx` and `src/features/chat/hooks/useResolvedAgentModelPicker.ts`

---

## Symptom

Open the **global composer pill** anywhere it's rendered (Home, sidebar, etc — anywhere outside an active chat). Click the **"Select model"** button in the bottom toolbar.

**Expected:** Popover opens upward and lists available models grouped by provider — same shape and population as the in-chat model picker (`AgentModelPicker`).

**Actual:** Popover opens upward (good — `side="top"` is set explicitly) but renders the `t("toolbar.noModelsAvailable")` empty state ("No models available"), even though:
- The user has providers configured in Settings
- Chat works fine — the in-chat `AgentModelPicker` shows models correctly in the same session
- Reloading the app does not help

The "No project" picker on the same pill works correctly — it lists projects and renders full-sized above the composer.

---

## Repro

1. Run `pnpm tauri dev` on `tulsi/visual-design`
2. Make sure at least one provider is configured in Settings (e.g. an Anthropic or OpenAI key)
3. Open Home (or any non-chat surface that renders `<GlobalComposerPill />`)
4. Click the **"Select model"** button at the bottom of the pill
5. Observe: popover above the pill says "No models available"
6. Open a chat session, click the in-chat model picker → models render correctly

---

## What I already tried (do not repeat)

These polish-level changes are committed in the same branch and should be **kept** — they're correct, just don't fix the data-flow issue:

1. **`focus-override`** added to the chat input className → fixes the white focus ring.
2. **`side="top"`** added to both `<PopoverContent>`s in the pill → both popovers now open upward by intent (no more reliance on Radix auto-flip with content-size dependency).
3. **Empty state** added to the model popover — `{modelGroups.length === 0 ? <p>{t("toolbar.noModelsAvailable")}</p> : null}` — defensive UX for the genuinely-empty case.
4. **On-open inventory sync** added to the pill — `useEffect` watching `modelPickerOpen` that calls `getProviderInventory()` from `@/features/providers/api/inventory` and merges into `useProviderInventoryStore`. **This is what `AgentModelPicker` does (lines 363–387) but it's NOT enough on its own** — see the hypothesis below.

If your fix supersedes the inventory-sync `useEffect`, feel free to remove it. The other three changes are visual polish unrelated to the bug.

---

## Hypothesis (probably wrong, but the place to start)

The chat path uses a **two-layer** model resolution:

- `useResolvedAgentModelPicker` (`src/features/chat/hooks/useResolvedAgentModelPicker.ts`) does substantial work the global pill skips:
  - Translates `selectedProvider` (an *agent ID* like `"goose"`) into a *provider catalog ID* via `resolveAgentProviderCatalogIdStrict`
  - Sources from `useProviderInventory()` *hook* (not the raw store) — see `src/features/providers/hooks/useProviderInventory.ts`
  - Maintains separate `gooseDefaultSelection` state
  - Ties model selection back to per-session model preferences via `acpSetModel`, `getStoredModelPreference`, etc.
- `AgentModelPicker` itself receives **fully-resolved** `agents`, `models`, `recommendedModels`, `currentModelId`, `currentModelName` props from `ChatInputToolbar`. It doesn't compute model groups; it just renders what the parent prepared.

By contrast, `GlobalComposerPill` **reads `useProviderInventoryStore.entries` directly** and tries to build groups inline via `buildModelGroups(providerInventoryEntries)`, filtering on `entry.models.length > 0`. If the store entries have `models: []` (which seems to be the case here), the filter eliminates them all → empty popover.

**Likely root cause candidates:**
1. The store has entries but their `models` arrays are empty in the global pill's read context — possibly because `refreshConfiguredProviderInventory` (in `src/app/hooks/useAppStartup.ts`, lines 88–135) populated the store but the polling didn't complete before the user opened the picker, or the model lists live in a different store / aren't merged into `useProviderInventoryStore`.
2. The chat picker doesn't read from `useProviderInventoryStore` for its model lists — it uses session-bound state plumbed via `useResolvedAgentModelPicker`. The global pill is reading the wrong source.
3. `selectedProvider` from `useAgentStore` is an *agent ID* like `"goose"` and the inventory store is keyed by *provider catalog ID*. `buildDefaultModelSelection` does some "if selectedProvider === 'goose' then enumerate all groups" handling, but `buildModelGroups` itself doesn't do agent → catalog resolution.

I don't know which of these is the actual cause — investigate before fixing.

---

## What I need

A model picker on `GlobalComposerPill` that:

1. **Lists the same models the in-chat `AgentModelPicker` lists** in the same session, grouped by provider, sorted with recommended models first.
2. **Updates when settings change** (a new provider is added/removed in Settings, or refresh kicks in).
3. **Persists the user's selection per send** — current behavior of `modelOverride` state on the pill is fine; what matters is that the underlying provider/model IDs match what the chat side considers valid.

If the cleanest path is to refactor the global pill to consume `useResolvedAgentModelPicker` (or a leaner variant of it), do that — don't preserve the current inline `buildModelGroups` approach if it's the wrong abstraction.

---

## Out of scope — don't change

- **Visual design** of the pill (`bottom-6 right-6`, `rounded-[40px]`, `bg-white/15` with the backdrop-filter glass effect, the bottom-toolbar layout, button shapes/sizes).
- **The empty state copy or styling** — `t("toolbar.noModelsAvailable")` wording is fine; if the picker actually has models post-fix, the empty branch just won't render.
- **The `side="top"` choice on both popovers** — that's intentional, both should open upward.
- **The `focus-override` addition** to the input — solves a separate global-CSS focus-ring issue.
- **The "No project" picker** — works correctly, leave it alone.

---

## Useful entry points

- `src/shared/ui/GlobalComposerPill.tsx` — the component that's broken
- `src/features/chat/ui/AgentModelPicker.tsx` — the picker that works (UI only — its data comes from below)
- `src/features/chat/hooks/useResolvedAgentModelPicker.ts` — the heavy lifting that prepares model data for the chat picker
- `src/features/chat/ui/ChatInputToolbar.tsx` — where `<AgentModelPicker>` is rendered with its resolved props
- `src/features/providers/stores/providerInventoryStore.ts` — the shared inventory store
- `src/features/providers/hooks/useProviderInventory.ts` — the hook the chat path uses (not the store directly)
- `src/features/providers/api/inventory.ts` — `getProviderInventory` (passive read) vs. `refreshProviderInventory` (active refresh)
- `src/app/hooks/useAppStartup.ts` lines 67–135 — startup-time inventory load + polling refresh
- `src/features/providers/providerCatalog.ts` — `resolveAgentProviderCatalogIdStrict` (agent ID → catalog ID resolution)

---

## Acceptance

- Open Home, click "Select model" in the global composer pill → see real models grouped by provider, identical to (or a sensible subset of) what the in-chat picker shows.
- Selecting a model and sending a message → message is sent with the chosen model.
- Adding a new provider in Settings while the pill is mounted → next time the model picker opens, the new provider/models show up.
- Type-check + lint clean.
- Don't break the chat-side picker.
