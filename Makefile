

release: target/release/devilnput


src/name_table.rs: scripts/gen-key-name-table.rb
	ruby scripts/gen-key-name-table.rb > src/name_table.rs


target/release/devilnput: src/name_table.rs src/main.rs
	cargo build --release
