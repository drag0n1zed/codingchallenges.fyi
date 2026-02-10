# drag's coding challenges

A collection of software (re-)written in Rust, following the roadmap from [codingchallenges.fyi](https://codingchallenges.fyi/). 

**Goal:** 1 challenge per week.

## Progress

| # | Challenge | Status |
| :--- | :--- | :--- |
| 01 | cc-wc | ✅ Done! |
| 02 | JSON Parser | 🚧 In Progress |
| 03 | Compression Tool | 📅 Pending |
| 04 | cut Tool | 📅 Pending |
| 05 | Load Balancer | 📅 Pending |

## Usage

This repository uses a Cargo workspace. To run a specific challenge:

```bash
cargo run -p <challenge_name> -- [args]
```

Example:
```bash
cargo run -p cc-wc -- -lcm test.txt
```
