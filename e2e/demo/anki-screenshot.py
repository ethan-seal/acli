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

def take_screenshot(output_path):
    """Take a screenshot using scrot"""
    result = subprocess.run(
        ['scrot', '-o', output_path],
        capture_output=True,
        timeout=10
    )
    return result.returncode == 0

def main():
    args = parse_args()
    
    # Ensure output directory exists
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    
    # Create collection directory if needed
    os.makedirs(args.collection, exist_ok=True)
    
    # Track if screenshot was taken
    screenshot_taken = [False]
    
    def screenshot_and_exit():
        """Take screenshot after a delay and exit"""
        time.sleep(args.timeout - 3)  # Wait for Browse to render
        if take_screenshot(args.output):
            print(f"Screenshot saved: {args.output}")
            screenshot_taken[0] = True
        else:
            print("ERROR: Failed to take screenshot")
        time.sleep(1)
        os._exit(0 if screenshot_taken[0] else 1)
    
    # Start screenshot thread
    screenshot_thread = Thread(target=screenshot_and_exit, daemon=True)
    screenshot_thread.start()
    
    # Import Anki modules
    from aqt import mw, gui_hooks
    from aqt.qt import QTimer
    from aqt.browser import Browser
    
    def open_browser_and_filter():
        """Open browser and filter to the specified deck"""
        print(f"Opening Browse window for deck: {args.deck}")
        if mw:
            # Open browser
            browser = mw.onBrowse()
            if browser:
                # Set search to filter by deck
                browser.search_for(f'deck:"{args.deck}"')
                print("Browse window opened and filtered")
            else:
                print("WARNING: Could not open browser")
    
    @gui_hooks.profile_did_open.append
    def on_profile_open():
        """Called when Anki profile is loaded"""
        print("Profile loaded, opening browser in 2 seconds...")
        QTimer.singleShot(2000, open_browser_and_filter)
    
    # Start Anki with the specified collection directory
    print(f"Starting Anki with collection: {args.collection}")
    sys.argv = ['anki', '-b', args.collection]
    
    from aqt import run
    run()

if __name__ == '__main__':
    main()
