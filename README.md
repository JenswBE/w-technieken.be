# Running locally

```bash
sudo apt install -y libssl-dev
cargo install cargo-watch
cargo watch -x run

# In another terminal
cd output
python3 -m http.server
```

# Update schema's

```bash
# Pull settings from local
npx directus-sync pull -c directus-sync/local.js

# Setup config for Prod
cp directus-sync/local.js directus-sync/prod.secret.js
editor directus-sync/prod.secret.js

# Compare local with Prod
npx directus-sync diff --debug -c directus-sync/prod.secret.js

# Push state to Prod
npx directus-sync push -c directus-sync/prod.secret.js
```

# Updating GraphQL schema

```bash
cargo install --locked cynic-cli
cynic introspect -H 'Authorization: Bearer MfoD5hw-BJbRbv7qUo4JU0zzxikOckJD' 'http://localhost:8055/graphql' -o schemas/directus.graphql
```
