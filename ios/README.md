# Commander Practice for iPad

Practice mode as an iPad app that plays offline. Inside the app is a copy of `commander_sim` (the same engine, AI,
decks and practice server as on your computer) and a web view showing the same table page. The server runs in the
app on a background thread and listens on the iPad only.

```
ios/
  pyproject.toml              the Briefcase project (app name, bundle id, iOS settings)
  stage.py                    copies the core, the deck lists and the card data into src/ before a build
  src/commander_ipad/
    app.py                    the app: starts the server, shows the table full screen
    bootstrap.py              start-up without Toga: data folders, first-launch copy (tests/test_ios_app.py)
    resources/                (staged, not in git) decklists/ and data/
  src/commander_sim/          (staged, not in git) the copy of the core
```

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
python3 ios/stage.py              # copy the current code, decks and card data into ios/src/
cd ios
briefcase dev                     # the app in a window on the Mac: the quickest check
briefcase create iOS              # first time only: makes the Xcode project
briefcase update iOS              # later times: puts the newly staged code into it
briefcase run iOS                 # builds it and starts it in the iPad simulator (pick an iPad)
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

- **Touch.** The page was made for a mouse: hover tips, small click targets, and the layout at iPad size.
- **Leaving the app.** iOS can close an app in the background, which loses a game in progress. The game should save
  itself when the app goes to the background, and offer to continue when it opens.
- **AI speed.** The look-ahead AI takes a few seconds per decision on a desktop; on the iPad it may need fewer
  playouts, or the adaptive AI by default.
- **Two-player mode.** The "me and a friend" option needs another computer on the network, so the app should hide it.
- **An app icon.**
