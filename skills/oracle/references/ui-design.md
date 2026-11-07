# UI that fits the project

Oracle uses Emil Kowalski's design engineering guidance as its UI craft foundation. Read the bundled [Design Engineering skill](emil-design-engineering.md) when building or reviewing components and interactions, and the [UI cheatsheet](emil-ui-cheatsheet.md) for small typography and feedback details. Read the relevant sections as needed rather than applying every animation recipe to every screen.

Oracle adds the project context: what the screen is for, how often someone uses it, what information must be visible together, and what evidence supports the result.

## Choose the layout, then refine the details

| Screen or task | Oracle context | Apply the shared foundation |
| --- | --- | --- |
| Portfolio, public page, or small informational site | [Simple-project guide](simple-projects.md): lead with real content and a clear next step | Deliberate type hierarchy, readable line lengths, useful imagery, restrained interaction feedback |
| Small utility or personal app | One short journey with clear input, result, correction, and persistence states | Labeled controls, stable layouts, visible focus, immediate feedback, and understandable saved/error states |
| Inventory, administration, or repeated comparison | [Compact UI guide](compact-ui.md): put relevant records and actions together | Aligned tabular numbers, quiet secondary chrome, visible selection, and fast repeated actions |
| Narrow screen or touch-oriented workflow | Preserve the task, essential values, and access to details | Larger usable targets, controls that reflow, no hover-only information, and explicit back/close actions |

Audience alone does not decide density. A public page can breathe while the same product's operations screen remains compact. An effective existing frontend stack stays in place; the upstream CSS and library examples are options, not dependencies to install automatically.

## Apply craft without slowing the task

- **Typography and hierarchy:** use consistent type roles, readable line lengths, and tabular numerals for changing quantities. Large headings can use tighter tracking. Keep secondary text legible; spacing and alignment do more work than a stack of decorative containers.
- **Controls and state:** make actions respond immediately. Show focus, selection, loading, success, validation, and recoverable failure where they occur. Keep labels and dimensions stable when button text changes. A saved message must reflect persistence, and a failed save should retain the draft.
- **Motion with a purpose:** follow the upstream frequency and purpose questions before choosing an animation. Repeated operations and keyboard navigation should stay immediate. Occasional transitions can explain a change; decorative motion must not block content or actions. Use targeted properties rather than `transition: all`.
- **Access across inputs:** honor reduced-motion preferences, preserve keyboard operation and visible focus, and gate hover-specific effects to appropriate inputs. Reduced motion may mean no animation; choose the least motion needed for comprehension. Keep controls usable without any animation.
- **Verify implementation details:** treat upstream durations and effects as starting points. Transform and opacity are useful performance candidates, not a guarantee of GPU compositing; profile the actual browser and device. Check current library APIs before copying snippets. A static illustration cannot verify motion, focus order, semantics, or persistence.

Use the upstream Before / After / Why table when reporting concrete UI findings. Keep the review proportional to the current work; a prototype needs a useful next action, not every possible polish task. Record the density decision under `UX-001` and actual workflow review under `UX-002`. No new gate or pixel budget is introduced.

## What the illustrations demonstrate

The three SVG examples are original Oracle illustrations, recreated using this foundation:

- [Simple projects](../assets/simple-projects.svg): editorial hierarchy and purposeful whitespace for a portfolio; a focused input, clear list state, and persistence copy for a small app.
- [Compact workspace](../assets/compact-workspace.svg): restrained navigation, an aligned toolbar, tabular quantities, selection, and a scoped action in the same view.
- [Adaptive workspace](../assets/adaptive-workspace.svg): a focused narrow view alongside a desktop editor that preserves list context, field focus, and a recoverable save error.

They contain fictional content. They are design references, not interactive components, upstream screenshots, or evidence that an application has passed usability or accessibility tests.

## Upstream source and license

Copied from [emilkowalski/skills](https://github.com/emilkowalski/skills) at revision [`c8a79627b59d72d764d6ab6f2c87c09a48169833`](https://github.com/emilkowalski/skills/tree/c8a79627b59d72d764d6ab6f2c87c09a48169833), retrieved 2026-11-07:

| Bundled file | Upstream path | Changes |
| --- | --- | --- |
| [emil-design-engineering.md](emil-design-engineering.md) | `skills/emil-design-eng/SKILL.md` | Filename only; contents unchanged |
| [emil-ui-cheatsheet.md](emil-ui-cheatsheet.md) | `ui-cheatsheet.md` | Filename only; contents unchanged |
| [emil-LICENSE.txt](emil-LICENSE.txt) | `LICENSE` | Filename only; contents unchanged |

Copyright (c) 2026 Emil Kowalski. Reused under the MIT license; the complete notice is bundled with every agent installation and source package. This page is Oracle's integration layer. Specialized upstream skills for other platforms and workflows remain available in the [upstream catalog](https://github.com/emilkowalski/skills/tree/main/skills); they are not automatically installed. The copy is pinned and works offline; upstream changes require an explicit review and update. No affiliation or endorsement is implied.
