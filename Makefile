# Versions
## https://github.com/softprops/lambda-rust
V_LAMBDA_RUST ?= "0.3.0-rust-1.45.0"
# Source files
## Find `Cargo.toml` files everywhere
CARGO_TOML := $(shell find . -name 'Cargo.toml')
## Files for the email_broker binary
BROKER=./email_broker
BROKER_SRC := $(shell find -E $(BROKER) -regex '.*\.rs')
## Files for the email_lambda binary
LAMBDA=./email_lambda
LAMBDA_SRC := $(shell find -E $(LAMBDA) -regex '.*\.rs') 
## Files for the email_shared lib
SHARED=./email_shared
SHARED_SRC := $(shell find -E $(SHARED) -regex '.*\.rs') 

.PHONY: all broker clean init lambda test

all: broker lambda

broker: target/release/email_broker

init: .git/config

lambda: target/release/email_lambda

clean:
	find infrastructure/cdk/bin -name '*.js' -or -name '*.d.ts' \
		| xargs -t rm
	find infrastructure/cdk/lib -name '*.js' -or -name '*.d.ts' \
		| xargs -t rm
	find infrastructure/cdk/test -name '*.js' -or -name '*.d.ts' \
		| xargs -t rm

test: $(SHARED_SRC) $(BROKER_SRC) $(LAMBDA_SRC)
	cargo test

.git/config: .githooks/*
	git config core.hooksPath .githooks

Cargo.lock: $(CARGO_TOML) init
	cargo check
	@touch -mr $(shell ls -Atd $? | head -1) $@

target/release/email_broker: Cargo.lock $(SHARED_SRC) $(BROKER_SRC)
	cargo build --release --bin email_broker

target/release/email_lambda: Cargo.lock $(SHARED_SRC) $(LAMBDA_SRC)
	cargo build --release --bin email_lambda

target/lambda/release/email_lambda: Cargo.lock $(SHARED_SRC) $(LAMBDA_SRC)
	docker run --rm \
		-e BIN=email_lambda \
		-e PACKAGE=false \
		-e PROFILE=release \
		-v ${PWD}:/code \
		-v ${HOME}/.cargo/registry:/root/.cargo/registry \
		-v ${HOME}/.cargo/git:/root/.cargo/git \
		softprops/lambda-rust:${V_LAMBDA_RUST}

target/lambda/runnable/fn:
	mkdir -p target/lambda/runnable/fn

target/lambda/runnable/fn/bootstrap: target/lambda/runnable/fn target/lambda/release/email_lambda
	cp target/lambda/release/email_lambda $@

# Build a zip archive which can be uploaded to AWS lambda
email_lambda.zip: target/lambda/release/email_lambda
	cp ./target/lambda/release/email_lambda ./bootstrap \
		&& zip email_lambda.zip bootstrap \
		&& rm bootstrap
