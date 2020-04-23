lambda.zip: target/x86_64-unknown-linux-musl/release/mutual-gf
	cp $< bootstrap
	strip bootstrap
	ls -l bootstrap
	rm -f lambda.zip
	zip lambda.zip bootstrap
	rm -f bootstrap

.PHONY: target/x86_64-unknown-linux-musl/release/mutual-gf
target/x86_64-unknown-linux-musl/release/mutual-gf:
	cargo build --bin mutual-gf --release --target x86_64-unknown-linux-musl

.PHONY: clean
clean:
	rm -f lambda.zip
	cargo clean
