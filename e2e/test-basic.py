#!/usr/bin/env python3
"""
Basic e2e test to verify Anki is working in the container.

Run with: ./e2e/run.sh --headless python /project/e2e/test-basic.py
"""
import sys
import os
import tempfile

def main():
    print("=" * 60)
    print("Anki E2E Test - Basic Functionality Check")
    print("=" * 60)
    
    # Test 1: Import anki modules
    print("\n[1] Testing Anki imports...")
    try:
        from anki.collection import Collection
        from anki.notes import Note
        from anki.decks import DeckId
        print("    OK: Core Anki modules imported successfully")
    except ImportError as e:
        print(f"    FAIL: Could not import Anki modules: {e}")
        return 1
    
    # Test 2: Create a collection
    print("\n[2] Testing collection creation...")
    try:
        with tempfile.TemporaryDirectory() as tmpdir:
            col_path = os.path.join(tmpdir, "test.anki2")
            col = Collection(col_path)
            print(f"    OK: Created collection at {col_path}")
            
            # Test 3: Create a deck
            print("\n[3] Testing deck creation...")
            deck_id = col.decks.id("TestDeck::SubDeck")
            deck = col.decks.get(deck_id)
            print(f"    OK: Created deck '{deck['name']}' with id {deck_id}")
            
            # Test 4: Get the Basic note type
            print("\n[4] Testing note type access...")
            model = col.models.by_name("Basic")
            if model is None:
                print("    FAIL: Basic note type not found")
                col.close()
                return 1
            print(f"    OK: Found note type '{model['name']}'")
            
            # Test 5: Create a note
            print("\n[5] Testing note creation...")
            note = col.new_note(model)
            note.fields[0] = "What is the capital of France?"
            note.fields[1] = "Paris"
            col.add_note(note, deck_id)
            print(f"    OK: Added note with id {note.id}")
            
            # Test 6: Query cards
            print("\n[6] Testing card query...")
            cards = col.find_cards("deck:TestDeck")
            print(f"    OK: Found {len(cards)} card(s) in deck")
            
            # Test 7: Verify card content
            print("\n[7] Verifying card content...")
            if cards:
                card = col.get_card(cards[0])
                note = col.get_note(card.nid)
                assert "France" in note.fields[0]
                assert "Paris" in note.fields[1]
                print(f"    OK: Card content verified")
                print(f"        Front: {note.fields[0]}")
                print(f"        Back: {note.fields[1]}")
            
            # Test 8: Save and close
            print("\n[8] Testing save...")
            col.close()
            print("    OK: Collection saved and closed")
            
    except Exception as e:
        print(f"    FAIL: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    print("\n" + "=" * 60)
    print("All tests passed!")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    sys.exit(main())
