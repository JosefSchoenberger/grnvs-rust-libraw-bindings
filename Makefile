# Copyright (c) 2023 Josef Schönberger
# 
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
# 
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
# 
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

CRATE_NAME=<assignment-name>

.PHONY:
build:

ifeq (1, $(ONLINE))
build: build_online
$(info Make: Building online...)
else
build: build_offline
endif

.PHONY:
all: build

common/cargo_deps.tar.gz: | common
	@mkdir -p .cargo
	cargo vendor cargo_deps >.cargo/config.toml
	tar -czf common/cargo_deps.tar.gz cargo_deps
	rm -r cargo_deps

cargo_deps: common/cargo_deps.tar.gz
	@printf "[ tar ] extracting Rust dependencies... (%s -> %s)\n" "$^" "$@"
	@tar -xf common/cargo_deps.tar.gz

.PHONY:
build_online: target/debug/deps/libraw.a
	@echo [cargo] build
	@cargo build
	@cp target/debug/${CRATE_NAME} .

.PHONY:
build_offline: target/debug/deps/libraw.a cargo_deps
	@echo [cargo] build --offline --frozen
	@cargo build --offline --frozen
	@cp target/debug/${CRATE_NAME} .

.PHONY:
clean:
	cargo clean
	rm -rf libraw/build
	rm -f ${CRATE_NAME}

.PHONY:
deepclean: clean
	rm -rf common cargo_deps .cargo
	rm -f Cargo.lock

libraw/build:
	@mkdir libraw/build

common:
	@mkdir common

libraw/build/%.o: libraw/src/%.c | libraw/build
	@printf "[ gcc ] compiling %-21s to %s\n" $< $@
	@gcc -c $< -o $@ -g -Wall -Wextra -O2 -fPIC -fno-strict-aliasing -Ilibraw/include

libraw/build/libraw.a: $(addprefix libraw/build/, $(patsubst %c,%o,$(notdir $(wildcard libraw/src/*.c))))
	@echo "[ ar  ] bundeling [" $^ ] into $@
	@ar -rcs $@ $^

target/debug/deps/libraw.a: libraw/build/libraw.a
	@echo "[ cp  ] copying libraw.a into cargo project"
	@mkdir -p target/debug/deps
	@cp ./libraw/build/libraw.a target/debug/deps
