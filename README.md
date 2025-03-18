# Runic-Jungle

AI Agents on Bitcoin powered by Internet Computer.

# TODOs
[ ] Withdrawal of Tokens.
[ ] Fetching Balances.
[ ] Generating Actions.

# Deployment guide
```bash
# for mac user
DOCKER_DEFAULT_PLATFORM=linux/amd64 ./start_docker.sh

# for linux
./start_docker.sh

dfx deploy internet_identity --argument '(null)'

cargo build --release --target wasm32-unknown-unknown -p runes_indexer

ollama serve

dfx deploy llm

# Set up and run the idempotent-proxy:

<!-- We use a modified version of idempotent-proxy that supports [Range requests](https://developer.mozilla.org/en-US/docs/Web/HTTP/Range_requests) to handle Bitcoin RPC responses that exceed the 2MB HTTPS outcall limit. -->

git clone https://github.com/octopus-network/idempotent-proxy
git checkout runes-indexer

cargo install --path src/idempotent-proxy-server

export USER=icp:test
export URL_LOCAL=http://127.0.0.1:18443
idempotent-proxy-server

# Test the proxy:
curl http://127.0.0.1:8080/URL_LOCAL \
  -H 'content-type:text/plain;' \
  -H 'idempotency-key: idempotency_key_001' \
  --data-binary '{"jsonrpc":"1.0","id":"curltext","method":"getblockhash","params":[0]}'

dfx deploy backend --argument '(record{
    commission_receiver = null;
    bitcoin_network = variant { regtest };
})'

dfx deploy frontend
```
