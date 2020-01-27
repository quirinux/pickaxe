



.PHONY: run build-all

CONF = pickaxe.yml

define build
	cargo build ${1}
endef

define run
	./target/${1}/pickaxe ./examples/${CONF}
endef

run: build-debug run-debug

run-debug:
	$(call run,debug)

build-debug:
	$(call build)

run-relase:
	$(call run,release)

build-release:
	$(call build, --release)

build-all: build-debug build-release

tree:
	exa -T --git --git-ignore
