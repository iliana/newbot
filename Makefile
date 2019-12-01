lambda.zip: target/x86_64-unknown-linux-musl/release/newbot
	cp $< bootstrap
	objcopy --compress-debug-sections bootstrap
	ls -l bootstrap
	rm -f lambda.zip
	zip lambda.zip bootstrap
	rm -f bootstrap

.PHONY: target/x86_64-unknown-linux-musl/release/newbot
target/x86_64-unknown-linux-musl/release/newbot:
	cargo build --bin newbot --no-default-features --release --target x86_64-unknown-linux-musl

.PHONY: clean
clean:
	rm -f lambda.zip
	cargo clean
