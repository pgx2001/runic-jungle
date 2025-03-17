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

dfx deploy backend --argument '(record{
    commission_receiver = null;
    bitcoin_network = variant { regtest };
})'

dfx deploy frontend
```
