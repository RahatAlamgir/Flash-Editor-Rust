# 07 · Presentations: Animation tab, slide transitions, slideshow

Asked for: when a project is a Presentation, an **Animation** tab appears where the user can add animations and transitions,
like PowerPoint. The user chose (2026-11-28): the project type is **saved in the file**; first version = **slide transitions,
slideshow playback, export to PowerPoint**. Object animations (entrance / emphasis / exit effects, the animation pane) were not
ticked for the first version: they are stage B below, to be confirmed.

## Stage A (this)

1. **Project type.** `Project.presentation: bool`, saved in `project.json` as `"ProjectType": "presentation"`. Set by
   New Project > type Presentation, by opening a template of type Presentation, and by File > "Use as presentation".
2. **Slide transition** per page: `Page.transition` (kind, direction, duration, advance on click, advance after N seconds).
   Saved per page in `project.json` (`"Transition"`), undoable with the page list. Kinds: None, Fade, Push, Wipe, Cover,
   Uncover, Zoom, Split. Direction for Push / Wipe / Cover / Uncover.
3. **Animation tab** (only for presentations): gallery of transitions, direction, duration, advance options,
   "Apply to all slides", Preview, "From start" (F5), "From this slide" (Shift+F5).
4. **Slideshow**: full screen, shows each page, plays its transition, click / Right / Space / Enter = next, Left / Backspace =
   back, Esc = stop, auto-advance after N seconds when set. Preview = the same, inside the window, for one transition.
5. **PowerPoint export** writes `<p:transition>` for every slide (fade, push, wipe, cover, pull, zoom, split), with speed and
   advance settings, in both the editable and the picture export.
6. The online Presentation templates are rebuilt with the project type (the build tool sets it from the catalog type).

## Stage B (done 2026-11-28, see the log)

Object animations: Entrance / Emphasis / Exit effects per object, start (On click / With previous / After previous), duration,
delay, an Animation pane (ordered list), preview, slideshow playback, `<p:timing>` export.

## Not in either stage

Sound, motion paths, triggers, Morph.
