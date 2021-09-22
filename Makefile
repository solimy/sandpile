all:
	cargo build --release
	mv target/release/sandpile .
