# Running locally

```bash
go install github.com/cespare/reflex@latest
reflex -s -vR 'output/.*' -R 'target/.*' -- cargo run
```
