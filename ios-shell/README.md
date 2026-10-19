# Commander Practice for iPad: the Rust version

The practice app on your iPad: a small Swift app with a web view, linking the Rust library
(`rust/crates/practice`). At launch the app starts the real practice server inside the app, which listens on the
iPad only (127.0.0.1), and shows its page full screen on true black. It is the same practice mode as
`commander-practice` on a computer (setup screen, table, hints, review, saving, *Continue*), with the Rust engine and
the look-ahead AI, and no copy of Python. It replaced the old Python (Briefcase) app.

```
ios-shell/
  App/                  the Swift app: start the server, show the page
  include/              the Rust library's C interface (practice.h) and its module map
  Assets.xcassets/      the app icon
  project.yml           the Xcode project, for XcodeGen
  build-rust.sh         builds the Rust library for iOS (build/PracticeCore.xcframework) and fills AppData/
  run.sh                build-rust.sh, then generate the project, build, and start it in the iPad simulator
  AppData/            (made by build-rust.sh, not in git) data/, decklists/, static/ that go into the app
```

What goes into the app (`AppData/`): `data/` (cards.json, decks.json, my_decks.toml, scryfall_cache.json,
printings/, and `bundle.json`, this build's stamp), `decklists/` and `static/` (the page). Card images are not in git:
if `data/images/` exists on the computer you build on, it is bundled too; otherwise the app downloads the images it
needs when the iPad is online, and draws a card from its name when it is not.

On first launch (and whenever a new build is installed) the app copies `AppData/data` and `decklists` into its own
writable folder (Application Support). Files are replaced by the new build's; images downloaded since stay. Saved
games and the autosave live in `Application Support/saves` and are never touched by a new build.

## Build it on your Mac

One-time setup:
1. Install **Xcode** from the App Store, open it once and accept the licence, and let it install the iOS
   simulator. Then in Terminal: `xcode-select --install`.
2. Install **Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` (the defaults are fine), then
   open a new Terminal window.
3. Install **XcodeGen**: `brew install xcodegen` (Homebrew: https://brew.sh).
4. Get the code: `git clone https://github.com/josephlauigan/commander-sim.git`, then `cd commander-sim` and
   `git checkout main`. If you already have it: `git fetch`, `git checkout main` and `git pull`.

Then, from the `commander-sim` folder:

```
ios-shell/run.sh
```

The first build takes a few minutes, because it compiles the Rust library three times (iPad, and simulators for
Apple silicon and Intel Macs). After that, the Simulator opens on an iPad Pro 13-inch (M5) with the app running: the
setup screen appears. To use another simulator, pass its name, for example
`DEVICE="iPad Air 11-inch (M3)" ios-shell/run.sh` (`xcrun simctl list devices` lists them).

## On your own iPad

1. Run `ios-shell/run.sh` once (it builds the library and generates `ios-shell/CommanderPractice.xcodeproj`). Do it
   again after every `git pull`.
2. Plug the iPad into the Mac with a cable and unlock it; tap *Trust* if it asks. (On the iPad, Settings › Privacy &
   Security › Developer Mode must be on: switch it on there and let it restart, if Xcode says so.)
3. Open `ios-shell/CommanderPractice.xcodeproj` in Xcode. In Xcode › Settings › Accounts, add your Apple ID (a free one
   is enough).
4. Click the *CommanderPractice* project in the left list, the *CommanderPractice* target, then *Signing &
   Capabilities*: tick *Automatically manage signing* and pick your Apple ID as *Team* (shown as "Your Name
   (Personal Team)"). If Xcode says the bundle identifier is taken, add something to the end of it, for example
   `com.josephlauigan.commander-ipad2`. (Regenerating the project with `run.sh` resets this, so the team may need
   picking again.)
5. At the top of the window choose your iPad as the run destination and press Run (the play button). The first build to
   a device takes a few minutes.
6. The first time, the iPad refuses to open the app ("Untrusted Developer"): on the iPad, Settings › General › VPN &
   Device Management › your Apple ID › Trust. Then open the app.

A free Apple ID signs the app for **7 days**. After that it won't open until you press Run in Xcode again (with the
iPad connected); your saved games stay, because the app is replaced, not deleted. A paid developer account ($99 a
year) makes it a year. Don't delete the app from the iPad to re-sign it: that deletes the saved games.

The app has the same bundle identifier the old Python (Briefcase) app had (`com.josephlauigan.commander-ipad`), so it
replaces that app in place. The old app's saved games don't carry over (they are in a different
format); if the iPad shows two apps, delete the old one.

## Checklist for the first run on the iPad

- The app opens full screen on black, no status bar; the setup screen lists your 12 decks with their win rates and
  the tiers.
- Start a game; play a few turns: tap a card in your hand, a land to tap it, a permanent for its abilities.
- Long press a card: it should enlarge. Check that touch targets feel right and nothing scrolls or zooms by accident.
- Try the AI speed setting; note whether the look-ahead AI makes you wait on your own turn, and how long.
- Close the app from the app switcher (swipe it up), reopen it: the setup screen should offer *Continue*, and it
  should put you back where you were.
- Mid-game, switch to another app (or lock the iPad) for 15 minutes or more, then come back: the table should still
  be there, or (if iOS stopped the server meanwhile) the page reloads and offers *Continue*.
- Turn the iPad (landscape and portrait); airplane mode: a game still plays (card pictures not yet downloaded are drawn
  from their names).

## What to tell me

- Whether `run.sh` finishes, and the first error if it doesn't. The likeliest problems are signing on a device,
  a simulator name that doesn't exist on your Mac, and XcodeGen not being installed.
- Anything on the checklist that doesn't work, on the simulator and on the iPad.
- The Swift part and the Xcode project have not been built yet (they were written on a Linux machine); expect the
  first run to maybe need a small fix.

## The two-player option

The page hides it in the app (as in the Briefcase app, which had no way to turn it on either). The library has
`practice_set_lan(true)`, which restarts the server on the local network on a new port, for a future gesture; nothing
in the app calls it.
