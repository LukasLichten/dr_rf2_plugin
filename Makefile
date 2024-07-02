.phony: all build run clean test help 

all: build
	cp target/release/libdr_rf2_plugin.so ../DataRace/plugins/

build:
	cargo build --release

run: all
	cd ../DataRace && ./target/release/launch_datarace

clean:
	cargo clean
	rm  ../DataRace/plugins/libdr_rf2_plugin.so

test: 
	# cargo test -p sample_plugin
	echo "TODO"

help:
	@echo "Makefile for build DataRace"
	@echo "make:             Builds the Plugin and copies it into the ../DataRace/plugins folder"
	@echo "make build:       Builds the Plugin"
	@echo "make run:         Runs DataRace"
	@echo "make clean:       Runs cargo clean and deletes the Plugin from ../DataRace/plugins/"
	@echo "make test:        TODO Runs tests on the plugin"
	@echo "make help:        Prints this info"
