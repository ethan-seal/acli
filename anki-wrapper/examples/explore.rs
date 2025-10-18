#[cfg(feature = "real-anki")]
fn main() {
    // Placeholder executable for exploring Anki APIs once the repo is populated under anki-wrapper/anki
    println!("anki-wrapper real-anki feature enabled; ready to explore.");
}

#[cfg(not(feature = "real-anki"))]
fn main() {
    println!("Build with --features real-anki after cloning Anki into anki-wrapper/anki");
}

