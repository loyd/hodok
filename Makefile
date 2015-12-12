-include config.mk

TARGET ?=
TOOLCHAIN ?=
LINKER ?=

RHOST ?=
RPATH ?=


export PATH := ./node_modules/.bin:$(PATH)

ifneq ($(TOOLCHAIN),)
	export LD_LIBRARY_PATH := $(TOOLCHAIN)/lib:$(LD_LIBRARY_PATH)
	export PATH := $(TOOLCHAIN)/bin:$(PATH)
endif
ifneq ($(TARGET),)
	CARGOFLAGS += --target=$(TARGET)
endif
ifneq ($(LINKER),)
	RUSTCFLAGS += -C linker="$(LINKER)"
endif

all: build/hodok build/index.html build/bundle.js build/assets

build/hodok: src/*.rs src/*/*.rs src/*/*/*.rs config.mk | build
	cargo rustc $(CARGOFLAGS) -- $(RUSTCFLAGS)
	cp target/$(TARGET)/debug/hodok $@

build/index.html: web/index.html | build
	cp $< $@

build/bundle.js: web/*.js web/tags/* | build
	browserify web/index.js -o $@

build/assets: web/assets/*
	mkdir -p $@
	touch $@
	cp $^ $@

build:
	mkdir -p $@

.PHONY: deploy remrun update clean

deploy: all
	rsync -vrE --delete --progress build/ $(RHOST):$(RPATH)

remrun: deploy
	ssh -t $(RHOST) 'cd $(RPATH) && ./hodok'

update: clean
	cargo update
	npm update

clean:
	rm -rf build
	cargo clean
