TARGET := target/arm-unknown-linux-gnueabihf/debug/rusty
CARGO := pi cargo

# TARGET := target/debug/rusty
# CARGO := cargo

BROWSERIFY := ./node_modules/.bin/browserify -d

RHOST := raspi@rusty
RPATH := /home/rusty

build: build/rusty build/index.html build/bundle.js

.PHONY: build/rusty
build/rusty: | mk-build
	$(CARGO) build
	cp $(TARGET) $@

build/index.html: web/index.html | mk-build
	cp $< $@

build/bundle.js: $(shell $(BROWSERIFY) --list web/index.js || echo '_') | mk-build
	$(BROWSERIFY) web/index.js -o $@

mk-build:
	mkdir -p build

deploy: build
	scp build/* $(RHOST):$(RPATH)

remrun: deploy
	ssh -t $(RHOST) 'cd $(RPATH) && ./rusty'

clean:
	rm -rf build
	cargo clean
