# emrent — Development Challenges 🛠️

A running log of real obstacles encountered while building emrent, written honestly.
Not a polished post-mortem — a journal of the actual fight, including every wrong
turn taken before the right answer showed up. If you're reading this, you're getting
the unfiltered version of what it took to build this thing. Grab a coffee. ☕

---

**Action 1 — The brotli war 🥊**
Nobody warns you that adding a desktop framework to a Rust project means inheriting
the entire dependency universe. When we scaffolded the Tauri workspace and ran
`cargo check` for the first time, we were greeted not by a clean compile but by a
cryptic failure deep inside a crate called `brotli` — a compression library that
nobody on the team explicitly asked for, pulled in silently as a transitive
dependency several levels deep in the tree. The error wasn't even about our code.
It was a version conflict between what Tauri needed and what brotli was shipping.
The first instinct was to run `cargo update` — let Cargo resolve it automatically.
That changed nothing. The second instinct was to check if the version constraints
in our own `Cargo.toml` were too strict. They weren't — we hadn't even specified
brotli ourselves, it was entirely a transitive dependency. We tried removing and
re-adding Tauri dependencies one at a time to isolate which one was dragging brotli
in. That was a rabbit hole with no end. We tried switching from `tauri = "2"` to
pinning an exact version in case a specific Tauri release had a cleaner dependency
resolution. Still nothing. What finally worked was going directly at the problem —
manually cloning the brotli source at a compatible version, dropping it into a
`patches/` directory at the workspace root, and telling Cargo to use our local
patched version instead of the one from crates.io via the `[patch.crates-io]`
directive in the root `Cargo.toml`. It compiled. But the lesson landed hard and
early — in a dependency tree of 620 crates, you are never truly in control of what
compiles and what doesn't, and sometimes the only way forward is to own the problem
yourself. Welcome to systems programming. 🦀

---

**Action 2 — The double builder that silently killed everything 👻**
Getting the Tauri dialog plugin to open a file picker should have been a thirty
minute job. It turned into one of those debugging sessions that makes you question
your life choices. The button was there, it was clickable, the app didn't crash —
just absolute silence when clicked, no file picker, no error, nothing. The first
thing we checked was the capabilities file at
`src-tauri/capabilities/default.json`. It was missing `dialog:allow-open` so we
added it, restarted the dev server and tried again. Still nothing. We then checked
whether `tauri-plugin-dialog` was properly listed in `Cargo.toml` under
`[dependencies]`. It was. We verified the JavaScript package
`@tauri-apps/plugin-dialog` was installed via npm. It was. We verified the versions
matched between the Rust crate and the JS package — `2.7.1` on both sides. They
matched. We tried calling the plugin differently in `App.tsx`, importing from
different paths, checking if the `open` function signature was correct. All of it
looked right. We restarted the dev server at least five more times convinced that
a stale build cache was the culprit. It wasn't. At some point we noticed that
`lib.rs` had two separate `tauri::Builder::default()` calls stacked one after the
other — a remnant of earlier edits where a second builder was added without removing
the first. The first builder was initializing correctly with the opener plugin and
running the application. Rust was happy, Tauri was happy, the window opened fine.
But the dialog plugin was registered on the *second* builder which never got
reached because the first one already consumed execution and the process never
returned from `.run()`. The app had been running perfectly on the wrong
configuration the entire time and neither the Rust compiler nor Tauri threw a
single warning about it. The fix was collapsing everything into one clean builder
with both plugins and both commands registered. Three lines of actual change after
hours of debugging. That's software. 😅

---

**Action 3 — window.__TAURI_INTERNALS__ and the browser trap 🕳️**
After fixing the double builder, a new wall appeared immediately. The frontend kept
throwing `Cannot read properties of undefined (reading 'invoke')` — a JavaScript
error meaning the Tauri IPC bridge, the thing that lets React talk to Rust, simply
didn't exist at runtime. The first fix attempted was adding `withGlobalTauri: true`
to the `app` section of `tauri.conf.json` — a documented Tauri configuration that
explicitly injects the IPC bridge into the webview as a global. That produced a
different error: `Additional properties are not allowed ('withGlobalTrust' was
unexpected)`. A typo — `withGlobalTrust` instead of `withGlobalTauri` — had
slipped in. Fixed the typo, restarted. The config error went away but the invoke
error remained. We then suspected a package version mismatch between the Rust
plugins and their JavaScript counterparts since this is a notoriously common Tauri
gotcha. We checked every `@tauri-apps/*` package version against its corresponding
Rust crate version. They all matched. During this investigation we discovered a
stray deprecated `tauri` npm package sitting in `package.json` outside the scripts
section — installed accidentally early on by running `npm install tauri` instead of
the correct `@tauri-apps/plugin-dialog`. We removed it, ran `npm install` again to
sync `node_modules`. The invoke error persisted. We cleared Vite's dev cache by
deleting `node_modules/.vite`. Still there. We added a `DOMContentLoaded` event
listener to `main.tsx` to log `window.__TAURI_INTERNALS__` directly and check if
the bridge was being injected at all. Ran the app, checked the console — and here
is where the real problem revealed itself. The console being checked this entire
time was the browser's console at `http://localhost:1420`, the Vite dev server URL,
not the actual Tauri native desktop window. When `npm run tauri dev` runs it starts
both a Vite dev server and a native desktop window. The browser and the native
window are completely separate processes. The browser will never have
`window.__TAURI_INTERNALS__` because it is not Tauri — it is just a browser. The
native Tauri window has its own built-in WebKit developer tools accessible by
pressing `F12` inside the window itself. When we finally opened the right console
inside the actual Tauri window, the error was completely different and pointed
directly back to the double builder from Action 2. An entire debugging session
consumed by looking at the wrong console. The browser and the Tauri window look
identical when running in development — same UI, same styles, same content — but
only one of them is actually Tauri. That distinction is easy to miss and expensive
when you do. 🎯