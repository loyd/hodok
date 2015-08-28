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

all: build/rusty build/index.html build/bundle.js

build/rusty: src/*.rs src/*/*.rs config.mk | build
	cargo rustc $(CARGOFLAGS) -- $(RUSTCFLAGS)
	cp target/$(TARGET)/debug/rusty $@

build/index.html: web/index.html | build
	cp $< $@

build/bundle.js: web/*.js web/tags/* | build
	browserify web/index.js -o $@

build:
	mkdir -p $@

.PHONY: deploy remrun update clean

deploy: all
	rsync -vrE --delete --progress build/ $(RHOST):$(RPATH)

remrun: deploy
	ssh -t $(RHOST) 'cd $(RPATH) && ./rusty'

update: clean
	cargo update
	npm update

clean:
	rm -rf build
	cargo clean
