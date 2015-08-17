-include config.mk

TARGET ?=
RUST ?=
LINKER ?=

RHOST ?=
RPATH ?=


export PATH := ./node_modules/.bin:$(PATH)

ifneq ($(RUST),)
	export LD_LIBRARY_PATH := $(RUST)/lib:$(LD_LIBRARY_PATH)
	export PATH := $(RUST)/bin:$(PATH)
endif
ifneq ($(TARGET),)
	CARGOFLAGS += --target=$(TARGET)
endif
ifneq ($(LINKER),)
	RUSTCFLAGS += -C linker="$(LINKER)"
endif

build: build/rusty build/index.html build/bundle.js

.PHONY: build/rusty
build/rusty: | mk-build
	cargo rustc $(CARGOFLAGS) -- $(RUSTCFLAGS)
	cp target/$(TARGET)/debug/rusty $@

build/index.html: web/index.html | mk-build
	cp $< $@

build/bundle.js: web/*.js | mk-build
	browserify web/index.js -o $@

mk-build:
	mkdir -p build

deploy: build
	rsync -vrE --delete --progress build/ $(RHOST):$(RPATH)

remrun: deploy
	ssh -t $(RHOST) 'cd $(RPATH) && ./rusty'

.PHONY: update
update: clean
	cargo update
	npm update

.PHONY: clean
clean:
	rm -rf build
	cargo clean
