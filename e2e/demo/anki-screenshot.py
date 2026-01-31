#!/usr/bin/env python3
"""
Take screenshots of Anki Browse window using the internal Python API.
This bypasses the keyboard shortcut issues in headless mode.

Usage:
    python anki-screenshot.py --collection /path/to/collection.anki2 --deck "DeckName" --output /path/to/screenshot.png
"""

import argparse
import os
import sys
import subprocess
import time
import json
from pathlib import Path
from threading import Thread

# Setup environment before importing Anki
os.environ.setdefault('QT_QPA_PLATFORM', 'xcb')

def parse_args():
    parser = argparse.ArgumentParser(description='Take Anki Browse screenshot')
    parser.add_argument('--collection', '-c', required=True, help='Path to collection directory')
    parser.add_argument('--deck', '-d', required=True, help='Deck name to filter')
    parser.add_argument('--output', '-o', required=True, help='Output screenshot path')
    parser.add_argument('--timeout', '-t', type=int, default=15, help='Timeout in seconds')
    return parser.parse_args()

def take_screenshot(output_path, window_id=None):
    """Take a screenshot using the best available method for the window ID"""
    if window_id:
        # Focus the window first
        subprocess.run(['xdotool', 'windowactivate', '--sync', str(window_id)],
                      capture_output=True, timeout=5)
        time.sleep(0.5)

        # Method 1: Use ImageMagick's import command with explicit window ID
        # This is the most reliable for headless environments
        result = subprocess.run(
            ['import', '-window', str(window_id), output_path],
            capture_output=True,
            timeout=15
        )
        if result.returncode == 0:
            return True

        print(f"  import command failed: {result.stderr.decode()}")

        # Method 2: Use xwd + convert (X Window Dump)
        xwd_result = subprocess.run(
            ['xwd', '-id', str(window_id), '-out', '/tmp/screenshot.xwd'],
            capture_output=True,
            timeout=10
        )
        if xwd_result.returncode == 0:
            convert_result = subprocess.run(
                ['convert', '/tmp/screenshot.xwd', output_path],
                capture_output=True,
                timeout=10
            )
            if convert_result.returncode == 0:
                return True
            print(f"  convert failed: {convert_result.stderr.decode()}")
        else:
            print(f"  xwd failed: {xwd_result.stderr.decode()}")

        # Method 3: Fallback to scrot focused window (less reliable in headless)
        print("  Falling back to scrot -u...")
        result = subprocess.run(
            ['scrot', '-u', '-o', output_path],
            capture_output=True,
            timeout=10
        )
        return result.returncode == 0
    else:
        # Full screen capture
        result = subprocess.run(
            ['scrot', '-o', output_path],
            capture_output=True,
            timeout=10
        )
        return result.returncode == 0


def find_window_by_name(name_pattern, min_width=None):
    """Find a window by name pattern (regex supported via xdotool --name)

    Args:
        name_pattern: Pattern to match window name (case-insensitive partial match)
        min_width: Optional minimum width to filter out small windows

    Returns:
        Window ID string or None
    """
    # Use --name with the pattern - xdotool does substring matching
    result = subprocess.run(
        ['xdotool', 'search', '--name', name_pattern],
        capture_output=True,
        timeout=5
    )
    if result.returncode != 0 or not result.stdout.decode().strip():
        return None

    window_ids = result.stdout.decode().strip().split('\n')

    # If no min_width filter, return the first match
    if min_width is None:
        return window_ids[0] if window_ids else None

    # Filter by window size
    for wid in window_ids:
        if not wid:
            continue
        geom_result = subprocess.run(
            ['xdotool', 'getwindowgeometry', wid],
            capture_output=True,
            timeout=5
        )
        if geom_result.returncode == 0:
            output = geom_result.stdout.decode()
            for line in output.split('\n'):
                if 'Geometry:' in line:
                    size = line.split(':')[1].strip()
                    try:
                        width = int(size.split('x')[0])
                        if width >= min_width:
                            return wid
                    except (ValueError, IndexError):
                        continue
    return None


def find_main_anki_window():
    """Find the main Anki window (not the small overlay)"""
    return find_window_by_name('Anki', min_width=500)

def clean_anki_locks(directory):
    """Remove stale SQLite WAL/SHM lock files that can cause Anki errors"""
    dir_path = Path(directory)
    lock_patterns = ['*.anki2-wal', '*.anki2-shm', '*.anki2-journal', '.lock']

    for pattern in lock_patterns:
        for lock_file in dir_path.glob(pattern):
            try:
                lock_file.unlink()
                print(f"Removed stale lock file: {lock_file}")
            except Exception as e:
                print(f"Warning: Could not remove {lock_file}: {e}")


def setup_anki_profile(base_dir):
    """
    Create Anki profile directory structure to skip first-run dialogs.
    This creates a 'User 1' profile that Anki will use automatically.

    Anki expects:
      base_dir/
        prefs21.db
        User 1/
          collection.anki2

    acli may create collection.anki2 at:
      - base_dir/collection.anki2 (older behavior)
      - base_dir/User 1/collection.anki2 (if profile already exists)

    Returns the profile directory path.
    """
    import shutil

    base_path = Path(base_dir)
    profile_dir = base_path / "User 1"
    profile_dir.mkdir(parents=True, exist_ok=True)

    # Clean up any stale lock files first - these cause "Anki encountered a problem"
    clean_anki_locks(base_path)
    clean_anki_locks(profile_dir)

    base_collection = base_path / "collection.anki2"
    profile_collection = profile_dir / "collection.anki2"

    # Check various locations for the collection
    collection_found = False

    if base_collection.exists():
        # Collection at base level - move it to profile
        print(f"Found collection at base: {base_collection}")
        if profile_collection.exists():
            # Remove existing profile collection (it's stale)
            print(f"Removing stale profile collection: {profile_collection}")
            profile_collection.unlink()
        shutil.copy2(str(base_collection), str(profile_collection))
        print(f"Copied collection to profile: {profile_collection}")
        # Keep the original for acli's use
        collection_found = True
    elif profile_collection.exists():
        # Collection already in profile directory
        print(f"Collection already in profile: {profile_collection}")
        collection_found = True
    else:
        # No collection found - this is OK for initial sync, Anki will create one
        print(f"No existing collection found at {base_collection} or {profile_collection}")
        print("Anki will create a new collection on first run")

    # Copy media database if it exists (don't move - acli may need it)
    media_db = base_path / "collection.media.db2"
    if media_db.exists():
        dest = profile_dir / media_db.name
        if not dest.exists():
            shutil.copy2(str(media_db), str(dest))
            print(f"Copied {media_db.name} to profile")

    # Create prefs21.db with proper profile configuration
    # This format was verified to work with Anki 24.11
    prefs_file = base_path / "prefs21.db"

    import sqlite3
    import pickle
    import random

    # Always recreate prefs to ensure correct structure
    if prefs_file.exists():
        prefs_file.unlink()

    conn = sqlite3.connect(str(prefs_file))
    cursor = conn.cursor()

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS profiles (
            name TEXT PRIMARY KEY COLLATE NOCASE,
            data BLOB NOT NULL
        )
    ''')

    # Create _global meta config - the format Anki 24.11 expects
    # CRITICAL: firstRun=False skips the first-run wizard
    meta_data = {
        'ver': 0,
        'updates': True,
        'created': int(time.time()),
        'id': random.randint(1000000000000000000, 9999999999999999999),
        'lastMsg': 0,
        'suppressUpdate': False,
        'firstRun': False,  # CRITICAL: Must be False to skip first-run dialog
        'defaultLang': 'en_US',
    }
    cursor.execute(
        'INSERT OR REPLACE INTO profiles (name, data) VALUES (?, ?)',
        ('_global', pickle.dumps(meta_data))
    )

    # Create profile entry for "User 1"
    profile_data = {
        'key': None,
        'created': int(time.time()),
    }
    cursor.execute(
        'INSERT OR REPLACE INTO profiles (name, data) VALUES (?, ?)',
        ('User 1', pickle.dumps(profile_data))
    )

    conn.commit()
    conn.close()
    print(f"Created profile preferences at {prefs_file}")
    
    return str(profile_dir)

def main():
    args = parse_args()
    
    # Ensure output directory exists
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    
    # Create collection directory and setup profile
    os.makedirs(args.collection, exist_ok=True)
    setup_anki_profile(args.collection)
    
    # Track state
    screenshot_taken = [False]
    browser_opened = [False]
    
    def screenshot_and_exit():
        """Take screenshot after browser opens, with retries"""
        # Wait for browser to open, polling more frequently
        wait_start = time.time()
        max_wait = args.timeout - 5  # Reserve 5 seconds for screenshot
        poll_interval = 0.3

        print(f"Waiting for browser to open (max {max_wait}s)...")
        while not browser_opened[0] and (time.time() - wait_start) < max_wait:
            time.sleep(poll_interval)

        if not browser_opened[0]:
            print("WARNING: Browser flag not set, continuing anyway...")

        # Check if Qt already took the screenshot
        if screenshot_taken[0]:
            print("Screenshot already taken via Qt, verifying file...", flush=True)
            if os.path.exists(args.output) and os.path.getsize(args.output) > 1000:
                print(f"Qt screenshot verified: {args.output} ({os.path.getsize(args.output)} bytes)")
                exit_code = 0
                print(f"Exiting with code {exit_code}...", flush=True)
                import signal
                os.kill(os.getpid(), signal.SIGTERM)
                return

        # Give time for rendering after browser opens
        # Use configurable delay based on timeout
        render_delay = min(4, args.timeout / 4)
        print(f"Waiting {render_delay}s for rendering...")
        time.sleep(render_delay)

        # Flush X11 events to ensure all pending rendering is complete
        try:
            subprocess.run(['xdotool', 'sync'], capture_output=True, timeout=5)
            print("X11 sync completed")
        except Exception as e:
            print(f"X11 sync warning: {e}")

        # Find the Browse window using multiple patterns
        # Anki's Browse title can be "Browse" or "Browse (deck:...)" or similar
        browse_wid = None
        browse_patterns = ['Browse', 'browse', 'Browser']

        for pattern in browse_patterns:
            browse_wid = find_window_by_name(pattern, min_width=400)
            if browse_wid:
                print(f"Found Browse window with pattern '{pattern}': {browse_wid}")
                break

        if not browse_wid:
            # Fall back to finding the largest Anki window
            browse_wid = find_main_anki_window()
            if browse_wid:
                print(f"Using main Anki window as fallback: {browse_wid}")
            else:
                print("WARNING: No suitable window found, trying full screen capture")

        # Attempt screenshot with retries
        max_attempts = 3
        for attempt in range(1, max_attempts + 1):
            print(f"Screenshot attempt {attempt}/{max_attempts}...")
            if take_screenshot(args.output, browse_wid):
                # Verify the file was created and has content
                if os.path.exists(args.output) and os.path.getsize(args.output) > 1000:
                    print(f"Screenshot saved: {args.output} ({os.path.getsize(args.output)} bytes)")
                    screenshot_taken[0] = True
                    break
                else:
                    print(f"Screenshot file too small or missing, retrying...")
            else:
                print(f"Screenshot attempt {attempt} failed")

            if attempt < max_attempts:
                time.sleep(1)

        if not screenshot_taken[0]:
            print("ERROR: All screenshot attempts failed", flush=True)

        exit_code = 0 if screenshot_taken[0] else 1
        print(f"Exiting with code {exit_code}...", flush=True)
        # Force immediate exit
        import signal
        os.kill(os.getpid(), signal.SIGTERM)
    
    # Start screenshot thread
    screenshot_thread = Thread(target=screenshot_and_exit, daemon=True)
    screenshot_thread.start()
    
    # Import Anki modules
    from aqt import mw, gui_hooks
    from aqt.qt import QTimer
    
    def open_browser_and_filter():
        """Open browser and filter to the specified deck"""
        import traceback
        # Re-import mw to get the actual instance after Anki starts
        from aqt import mw as main_window
        print(f"Opening Browse window for deck: {args.deck}", flush=True)
        print(f"mw = {main_window}", flush=True)

        def mark_browser_ready():
            """Called after delay to allow search results to populate"""
            # Try to get card count for debugging
            try:
                if main_window and main_window.col:
                    card_count = main_window.col.card_count()
                    print(f"Collection has {card_count} total cards", flush=True)
                    # Check deck
                    decks = main_window.col.decks.all_names_and_ids()
                    print(f"Decks: {[d.name for d in decks]}", flush=True)
            except Exception as e:
                print(f"Could not get card count: {e}", flush=True)
            print("Browser search should be complete, marking ready for screenshot", flush=True)
            browser_opened[0] = True

        try:
            if main_window:
                # Open browser - onBrowse returns the browser but may need time
                print("Calling mw.onBrowse()...", flush=True)
                browser = main_window.onBrowse()
                print(f"onBrowse returned: {type(browser)} - {browser}", flush=True)

                if browser:
                    # Set search to filter by deck
                    try:
                        search_query = f'deck:"{args.deck}"'
                        print(f"Searching for: {search_query}", flush=True)
                        browser.search_for(search_query)
                        # Resize window to show more content
                        browser.resize(1400, 900)
                        print("Browse window opened and filtered, waiting for search results...", flush=True)

                        def force_primary_render():
                            """Force the table to render when using primary browser path"""
                            from aqt.qt import QApplication
                            try:
                                table_view = browser.table._view
                                model = browser.table._model
                                if model:
                                    row_count = model.rowCount()
                                    print(f"Primary path: Table has {row_count} rows", flush=True)
                                    if row_count > 0:
                                        from aqt.qt import QModelIndex
                                        model.dataChanged.emit(model.index(0, 0), model.index(row_count - 1, model.columnCount() - 1))
                                if table_view:
                                    table_view.scrollToTop()
                                    table_view.selectRow(0)
                                    table_view.viewport().update()
                                    table_view.repaint()
                                browser.repaint()
                                QApplication.processEvents()
                                QApplication.processEvents()
                                print("Primary path: Forced render complete", flush=True)
                            except Exception as e:
                                print(f"Primary path render error: {e}", flush=True)

                        def open_primary_preview():
                            """Open card preview in primary browser path"""
                            from aqt.qt import QApplication
                            try:
                                if hasattr(browser, 'onTogglePreview'):
                                    browser.onTogglePreview()
                                    print("Primary path: Opened preview via onTogglePreview()", flush=True)
                                elif hasattr(browser, '_on_preview'):
                                    browser._on_preview()
                                    print("Primary path: Opened preview via _on_preview()", flush=True)
                                elif hasattr(browser, 'form') and hasattr(browser.form, 'actionToggle_Preview'):
                                    browser.form.actionToggle_Preview.trigger()
                                    print("Primary path: Opened preview via actionToggle_Preview", flush=True)
                                elif hasattr(browser, 'togglePreview'):
                                    browser.togglePreview()
                                    print("Primary path: Opened preview via togglePreview()", flush=True)
                                else:
                                    print("Primary path: No preview method found", flush=True)
                                QApplication.processEvents()
                                QApplication.processEvents()
                            except Exception as e:
                                print(f"Primary path: Could not open preview: {e}", flush=True)

                        def take_primary_screenshot():
                            """Take screenshot in primary browser path"""
                            from aqt.qt import QApplication
                            try:
                                QApplication.processEvents()
                                QApplication.processEvents()
                                pixmap = browser.grab()
                                if pixmap and not pixmap.isNull():
                                    saved = pixmap.save(args.output, "PNG")
                                    if saved:
                                        print(f"Primary path: Screenshot saved: {args.output}", flush=True)
                                        screenshot_taken[0] = True
                                    else:
                                        print("Primary path: Screenshot save failed", flush=True)
                                else:
                                    print("Primary path: grab() returned null pixmap", flush=True)
                            except Exception as e:
                                print(f"Primary path: Screenshot error: {e}", flush=True)

                        # Delay to allow search, then force render, open preview, screenshot, then mark ready
                        QTimer.singleShot(1500, force_primary_render)
                        QTimer.singleShot(2000, open_primary_preview)
                        QTimer.singleShot(2500, take_primary_screenshot)
                        QTimer.singleShot(3500, mark_browser_ready)
                    except Exception as e:
                        print(f"Error configuring browser: {e}", flush=True)
                        traceback.print_exc()
                else:
                    # Try alternative method - import and create browser directly
                    print("Trying alternative browser open method...", flush=True)
                    try:
                        from aqt.browser.browser import Browser
                        browser = Browser(main_window)
                        # First show all cards to populate the table
                        print("Showing all cards first...", flush=True)
                        browser.search_for("")  # Empty search shows all
                        browser.resize(1400, 900)
                        browser.show()

                        # Then filter to the specific deck after a short delay
                        def apply_deck_filter():
                            search_query = f'deck:"{args.deck}"'
                            print(f"Now filtering to: {search_query}", flush=True)
                            browser.search_for(search_query)
                            # Check how many cards match
                            try:
                                model = browser.table._model
                                if model:
                                    row_count = model.rowCount()
                                    print(f"Browser showing {row_count} cards", flush=True)
                            except Exception as e:
                                print(f"Could not get row count: {e}", flush=True)

                        def select_first_card():
                            """Select the first card to show it in preview"""
                            try:
                                # Try to select the first row
                                browser.table.select_single_card(browser.table.get_card_ids()[0])
                                print("Selected first card", flush=True)
                            except Exception as e:
                                print(f"Could not select first card: {e}", flush=True)
                                # Try alternative: use key press
                                try:
                                    browser.table._view.selectRow(0)
                                    print("Selected row 0 via view", flush=True)
                                except Exception as e2:
                                    print(f"Alternative selection failed: {e2}", flush=True)

                        def open_card_preview():
                            """Open the card preview panel to show front/back of selected card"""
                            from aqt.qt import QApplication
                            try:
                                # Method 1: Try the modern sidebar preview (Anki 2.1.x)
                                if hasattr(browser, 'onTogglePreview'):
                                    browser.onTogglePreview()
                                    print("Opened preview via onTogglePreview()", flush=True)
                                # Method 2: Try _on_preview (some versions)
                                elif hasattr(browser, '_on_preview'):
                                    browser._on_preview()
                                    print("Opened preview via _on_preview()", flush=True)
                                # Method 3: Try toggling via action (menu item)
                                elif hasattr(browser, 'form') and hasattr(browser.form, 'actionToggle_Preview'):
                                    browser.form.actionToggle_Preview.trigger()
                                    print("Opened preview via actionToggle_Preview", flush=True)
                                # Method 4: Try the sidebar toggle (Anki 24.x)
                                elif hasattr(browser, 'togglePreview'):
                                    browser.togglePreview()
                                    print("Opened preview via togglePreview()", flush=True)
                                # Method 5: Access previewer directly
                                elif hasattr(browser, '_previewer'):
                                    if browser._previewer is None:
                                        from aqt.browser.previewer import BrowserPreviewer
                                        browser._previewer = BrowserPreviewer(browser, browser.mw, lambda: browser.table.get_current_card())
                                        browser._previewer.open()
                                        print("Created and opened BrowserPreviewer", flush=True)
                                    else:
                                        browser._previewer.open()
                                        print("Opened existing previewer", flush=True)
                                else:
                                    # Try keyboard shortcut simulation as last resort
                                    from aqt.qt import QKeyEvent, Qt, QCoreApplication
                                    # Ctrl+Shift+P is typically preview shortcut
                                    event = QKeyEvent(QKeyEvent.Type.KeyPress, Qt.Key.Key_P, 
                                                     Qt.KeyboardModifier.ControlModifier | Qt.KeyboardModifier.ShiftModifier)
                                    QCoreApplication.postEvent(browser, event)
                                    print("Sent Ctrl+Shift+P key event for preview", flush=True)
                                
                                QApplication.processEvents()
                                QApplication.processEvents()
                                print("Card preview opened", flush=True)
                            except Exception as e:
                                print(f"Could not open card preview: {e}", flush=True)
                                import traceback
                                traceback.print_exc()

                        def force_table_render():
                            """Force the table to fully render its contents"""
                            from aqt.qt import QApplication
                            try:
                                # Get the table view
                                table_view = browser.table._view
                                model = browser.table._model

                                if model:
                                    row_count = model.rowCount()
                                    col_count = model.columnCount()
                                    print(f"Table has {row_count} rows, {col_count} columns", flush=True)

                                    # Force model to emit dataChanged for all cells
                                    if row_count > 0 and col_count > 0:
                                        from aqt.qt import QModelIndex
                                        top_left = model.index(0, 0)
                                        bottom_right = model.index(row_count - 1, col_count - 1)
                                        model.dataChanged.emit(top_left, bottom_right)
                                        print("Emitted dataChanged signal", flush=True)

                                # Force viewport update
                                if table_view:
                                    viewport = table_view.viewport()
                                    if viewport:
                                        viewport.update()
                                        print("Updated table viewport", flush=True)

                                    # Scroll to top to ensure first rows are visible
                                    table_view.scrollToTop()
                                    print("Scrolled table to top", flush=True)

                                    # Force table repaint
                                    table_view.repaint()
                                    print("Repainted table view", flush=True)

                                # Force browser window repaint
                                browser.repaint()

                                # Process all pending Qt events to ensure rendering completes
                                QApplication.processEvents()
                                QApplication.processEvents()  # Double-pump to be safe
                                print("Processed Qt events", flush=True)

                            except Exception as e:
                                print(f"Error forcing table render: {e}", flush=True)
                                import traceback
                                traceback.print_exc()

                        def take_qt_screenshot():
                            """Take screenshot using Qt's native grab() method"""
                            from aqt.qt import QApplication
                            try:
                                # Ensure all painting is complete
                                QApplication.processEvents()
                                QApplication.processEvents()

                                # Use Qt's grab() to capture the widget directly
                                pixmap = browser.grab()
                                if pixmap and not pixmap.isNull():
                                    # Save with high quality
                                    saved = pixmap.save(args.output, "PNG")
                                    if saved:
                                        print(f"Qt native screenshot saved: {args.output}", flush=True)
                                        screenshot_taken[0] = True
                                        # Signal that we're done
                                        browser_opened[0] = True
                                    else:
                                        print("Qt screenshot save failed", flush=True)
                                else:
                                    print("Qt grab() returned null pixmap", flush=True)
                            except Exception as e:
                                print(f"Qt screenshot error: {e}", flush=True)
                                import traceback
                                traceback.print_exc()

                        QTimer.singleShot(500, apply_deck_filter)
                        QTimer.singleShot(1500, select_first_card)
                        QTimer.singleShot(2000, open_card_preview)
                        QTimer.singleShot(2500, force_table_render)
                        # Take Qt screenshot after preview is open and rendered
                        QTimer.singleShot(3000, take_qt_screenshot)
                        print("Browser opened via alternative method, waiting for search results...", flush=True)
                        # Delay marking ready to allow screenshot to complete
                        QTimer.singleShot(4000, mark_browser_ready)
                    except Exception as e:
                        print(f"Alternative method failed: {e}", flush=True)
                        traceback.print_exc()
                        print("WARNING: Could not open browser", flush=True)
            else:
                print("ERROR: mw is still None after re-import!", flush=True)
        except Exception as e:
            print(f"EXCEPTION in open_browser_and_filter: {e}", flush=True)
            traceback.print_exc()
    
    @gui_hooks.profile_did_open.append
    def on_profile_open():
        """Called when Anki profile is loaded"""
        print("Profile loaded, opening browser in 1 second...")
        QTimer.singleShot(1000, open_browser_and_filter)
    
    # Start Anki with the specified collection directory, language, and safemode
    # Safemode is required for proper window initialization in headless mode
    print(f"Starting Anki with collection: {args.collection}")
    print(f"Profile directory: {args.collection}/User 1")

    # Check what files exist before starting
    profile_dir = Path(args.collection) / "User 1"
    print(f"Files in profile dir: {list(profile_dir.glob('*')) if profile_dir.exists() else 'N/A'}")

    sys.argv = ['anki', '-b', args.collection, '-l', 'en', '-p', 'User 1', '--safemode']

    # Enable more verbose logging
    os.environ['ANKI_DEBUG'] = '1'

    from aqt import run
    run()

if __name__ == '__main__':
    main()
