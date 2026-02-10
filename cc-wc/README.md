# cc-wc

Only supports single files. 

If counting chars and invalid UTF-8 is found, a warning is written to stderr.

Use:
```bash
cargo run -- test/test.txt
```

Compare with GNU `wc` and uutils `uu-wc`:
```bash
cargo install --path .
hyperfine 'cc-wc -lcmw test/test.txt' 'wc -lcmw test/test.txt' 'uu-wc -lcmw test/test.txt' -N
```

`cc-wc` is faster than `uu-wc` because it has less features.
`cc-wc` is faster than `wc` because Rust is better than C. And it has less features.
