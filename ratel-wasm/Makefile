target = wasm32-unknown-emscripten
output_path = ../target/wasm32-unknown-emscripten/release

static_files = $(wildcard client-src/*.js) \
       		   $(wildcard client-src/*.css) \
       		   $(wildcard client-src/*.ico) \
			   $(wildcard client-src/*.html) \
       		   $(wildcard client-src/iconfont)

all: clean build link assets

build:
	cargo build --release --target $(target)

clean:
	rm -rf $(output_path)/ratel_wasm*

link:
	(cd $(output_path)/ ; ln -s deps/ratel_wasm-*.wasm ./)

serve:
	cd $(output_path)/ && python -m SimpleHTTPServer

assets: $(static_files)
	cp -r $^ $(output_path)/
