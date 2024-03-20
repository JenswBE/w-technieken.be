# Running locally

```bash
sudo apt install -y libssl-dev
cargo install cargo-watch
cargo watch -x run

# In another terminal
cd output
python3 -m http.server
```

# Updating GraphQL schema

```bash
cargo install --locked cynic-cli
cynic introspect -H 'Authorization: Bearer MfoD5hw-BJbRbv7qUo4JU0zzxikOckJD' 'http://localhost:8055/graphql' -o schemas/directus.graphql
```
