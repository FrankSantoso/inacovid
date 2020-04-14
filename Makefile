INSTALLDIR?=${HOME}/.local/bin/

.PHONY: install

build:
	cargo build --release

install: build
	install -Dm755 target/release/inacovid ${INSTALLDIR}