# Commander Practice for iPad

Practice mode as an iPad app that plays offline. Inside the app is a copy of `commander_sim` (the same engine, AI,
decks and practice server as on your computer) and a web view showing the same table page. The server runs in the
app on a background thread and listens on the iPad only.

```
ios/
  pyproject.toml              the Briefcase project (app name, bundle id, iOS settings)
  icons/                      the app icon at every size (python3 tools/app_icon.py redraws them)
  stage.py                    copies the core, the deck lists and the card data into src/ before a build
  run.sh                      stage, build and start in the iPad simulator (the Mac)
  src/commander_ipad/
    app.py                    the app: starts the server, shows the table full screen
    bootstrap.py              start-up without Toga: data folders, first-launch copy (tests/test_ios_app.py)
    resources/                (staged, not in git) decklists/ and data/
  src/commander_sim/          (staged, not in git) the copy of the core
```

**Leaving the app.** iOS may close the app while it's in the background. The game saves itself each time it waits
on you, so when the app opens again the setup screen offers *Continue* for the game you were in. The app also hides
the two-player option, which needs a second computer.

**Where files go.** An app's bundle is read-only. The app copies the bundled `data/` (the card cache and the card
images) into its own writable folder the first time each build runs; saved games and images downloaded on the iPad
live there too and survive updates. A new build's card cache always replaces the old one.

## Build it on your Mac

One-time setup:
1. Install **Xcode** from the App Store, open it once, and accept the licence. Then in Terminal:
   `xcode-select --install`.
2. Install Python 3.12 or 3.13 (from python.org, or `brew install python@3.12`). Briefcase builds the app with the
   Python version that runs it.
3. Get the code and Briefcase:
   ```
   git clone https://github.com/josephlauigan/commander-sim.git
   cd commander-sim
   git checkout ios-app
   python3 -m venv .venv
   source .venv/bin/activate
   pip install briefcase
   ```
4. Download every card image, so the app has them offline (once; it takes a while, and later runs only fetch new
   cards):
   ```
   python3 ios/stage.py --images
   ```

Then each time:
```
ios/run.sh                        # stage, update the app, build, and start it in the iPad Pro 13-inch (M5) simulator
```
`DEVICE="iPad Air 11-inch (M3)" ios/run.sh` picks another simulator (`xcrun simctl list devices` lists them). The
steps it runs, if you want them one at a time:
```
python3 ios/stage.py              # copy the current code, decks and card data into ios/src/
cd ios
briefcase dev                     # the app in a window on the Mac: the quickest check
briefcase run iOS -u -d "iPad Pro 13-inch (M5)"    # -u: put the newly staged code in first (creates the app the first time)
```

## Put it on your iPad

1. `briefcase open iOS` opens the project in Xcode.
2. In Xcode, select the app target, open **Signing & Capabilities**, and under **Team** pick your Apple ID (add it
   under Xcode > Settings > Accounts if it isn't there).
3. Connect the iPad by cable. On the iPad turn on **Settings > Privacy & Security > Developer Mode** (it restarts).
4. Choose the iPad as the run destination at the top of Xcode and press Run.
5. The first time, the iPad asks you to trust the developer: **Settings > General > VPN & Device Management**.

With a free Apple ID the app stops opening after 7 days; connect the iPad and press Run again. The paid Apple
Developer Program ($99 a year) makes installs last a year and allows TestFlight, to share the app with JD and Avery.
Keep it off the public App Store: the card images are Wizards of the Coast's.

If Briefcase says a `toga-iOS` version can't be found, change the version in `pyproject.toml` to the current one
(`pip index versions toga-iOS`) and run `briefcase create iOS` again.

## Not done yet

- **Touch, on a real iPad.** Done so far, checked in screenshots at iPad sizes but not by hand: a long press
  enlarges a card (a tap still plays it), no text selection or system menu on the table, no double-tap zoom, bigger
  buttons, and a compact header in landscape so the boards and hand keep their room. Still to try on the device:
  how the long press and the ability menus feel, and whether cards need to be bigger (A+ in the header does that).
- **AI speed.** The look-ahead AI takes a few seconds per decision on a desktop; on the iPad it may need fewer
  playouts, or the adaptive AI by default.
