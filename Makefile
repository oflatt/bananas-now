.PHONY: debug install build serve

serve:
	cargo watch --shell "make debug && python3 -m http.server 8000 -d ${WWW}"


serve-andrey:
	cargo watch --shell "make debug && python3 -m http.server 8000 -d ${WWW}"

debug-andrey:
	cargo build --target wasm32-unknown-unknown
	wasm-bindgen --no-typescript --target web \
			--out-dir target/www \
			--out-name "mygame" \
			./target/wasm32-unknown-unknown/debug/bevy-hello-world.wasm



debug:
	cargo build --target wasm32-unknown-unknown
	rm -rf ${WWW} || true
	wasm-bindgen --no-typescript --target web \
			--out-dir target/www \
			--out-name "mygame" \
			./target/wasm32-unknown-unknown/debug/bevy-hello-world.wasm
	cp index.html ${WWW}/index.html
	cp -r assets ${WWW}/assets

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
	cargo install cargo-watch
