build:
	cargo build --target wasm32-unknown-unknown --release
clean:
	cargo clean
d-acc:
	near delete crosscall.bulksender.testnet bulksender.testnet
c-acc:
	near create-account crosscall.bulksender.testnet --masterAccount bulksender.testnet
deploy:
	near deploy crosscall.bulksender.testnet target/wasm32-unknown-unknown/release/bulk_sender.wasm
init:
	near deploy crosscall.bulksender.testnet target/wasm32-unknown-unknown/release/bulk_sender.wasm --initFunction new --initArgs '{"oracle_id": "oracle.bulksender.testnet"}'