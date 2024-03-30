.PHONY: debug install build

debug:
	cargo build --target wasm32-unknown-unknown
	wasm-bindgen --no-typescript --target web \
			--out-dir ./out/ \
			--out-name "mygame" \
			./target/wasm32-unknown-unknown/debug/bevy-hello-world.wasm
	cp index.html out/index.html
	cp -r assets out/assets

build:
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --no-typescript --target web \
			--out-dir ./out/ \
			--out-name "mygame" \
			./target/wasm32-unknown-unknown/release/mygame.wasm
	cp index.html out/index.html
	cp -r assets out/assets

install:
	rustup target install wasm32-unknown-unknown
	cargo install wasm-server-runner
	cargo install wasm-bindgen-cli
