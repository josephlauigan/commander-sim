# Commander Practice for iPad: the Rust version (M6 proof)

The iPad app as it will be after the Rust port: a small Swift app with a web view, linking the Rust library
(`rust/crates/practice`). At launch the app starts the server inside the app, which listens on the iPad only
(127.0.0.1), and shows its page full screen on true black. This replaces the Briefcase app in `ios/`, which
carries a copy of Python.

For now, the page is the proof that the pieces work on the iPad. *Play a game* plays Sauron against three Tier 1
decks with the Rust engine and the look-ahead AI, using the card data bundled in the app, and shows who won and how
long the game took. The practice table, saving, *Continue* and the two-player option come with the port's
practice-mode phases.

```
ios-shell/
  App/                  the Swift app: start the server, show the page
  include/              the Rust library's C interface (practice.h) and its module map
  project.yml           the Xcode project, for XcodeGen
  build-rust.sh         builds the Rust library for iOS (build/PracticeCore.xcframework), copies the card data
  run.sh                build-rust.sh, then generate the project, build, and start it in the iPad simulator
```

## Build it on your Mac

One-time setup:
1. Install **Xcode** from the App Store, open it once and accept the licence, and let it install the iOS
   simulator. Then in Terminal: `xcode-select --install`.
2. Install **Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` (the defaults are fine), then
   open a new Terminal window.
3. Install **XcodeGen**: `brew install xcodegen` (Homebrew: https://brew.sh).
4. Get the code: `git clone https://github.com/josephlauigan/commander-sim.git`, then `cd commander-sim` and
   `git checkout rust-port`. If you already have it: `git fetch` and `git checkout rust-port`, then `git pull`.

Then, from the `commander-sim` folder:

```
ios-shell/run.sh
```

The first build takes a few minutes, because it compiles the Rust library three times (iPad, and simulators for
Apple silicon and Intel Macs). After that, the Simulator opens on an iPad Pro 13-inch (M5) with the app running. Tap
*Play a game*: a result should appear after a second or two. To use another simulator, pass its name, for example
`DEVICE="iPad Air 11-inch (M3)" ios-shell/run.sh` (`xcrun simctl list devices` lists them).

**On your iPad.** After `run.sh` has run once, open `ios-shell/CommanderPractice.xcodeproj` in Xcode. Choose the
CommanderPractice target, and under *Signing & Capabilities* pick your Apple ID as the team. Then pick your iPad
as the run destination and press Run. The first time, the iPad asks you to trust the developer under
*Settings › General › VPN & Device Management*.

## What to tell me

- Whether `run.sh` finishes, and the first error if it doesn't. The likeliest problems are signing on a device,
  a simulator name that doesn't exist on your Mac, and XcodeGen not being installed.
- Whether *Play a game* shows a result in the simulator, and roughly how long it took (the time is on the page).
- The same on the iPad, if you try it there.
