# `wotbreplay-inspector`

World of Tanks Blitz replay inspector in Rust. Uses [`eigenein/wotbreplay-parser`](https://github.com/eigenein/wotbreplay-parser) under the hood.

[![Crates.io](https://img.shields.io/crates/v/wotbreplay-inspector)](https://crates.io/crates/wotbreplay-inspector)
[![Last commit](https://img.shields.io/github/last-commit/eigenein/wotbreplay-inspector)](https://github.com/eigenein/wotbreplay-inspector/commits/main)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/eigenein/wotbreplay-inspector/Check)](https://github.com/eigenein/wotbreplay-inspector/actions)
![License: MIT](https://img.shields.io/crates/l/wotbreplay-inspector)

## Quickstart

### Dump reverse-engineered fields as per `wotbreplay-parser`

```
❯ wotbreplay-inspector 20221205_1409__zeekrab_A140_ASTRON_REX_105_2308651318200102307.wotbreplay battle-results
{
  "timestamp": 1670245795,
  "players": [
    {
      "account_id": 534505602,
      "info": {
        "nickname": "Roberto_Cadenas_Diaz",
        "platoon_id": null,
        "team_number": 2,
        "clan_tag": "ORUGA"
…
```

### Dump full raw structure

```
❯ wotbreplay-inspector 20221205_1409__zeekrab_A140_ASTRON_REX_105_2308651318200102307.wotbreplay battle-results --raw
[
  {
    "tag": 1,
    "value": {
      "VarInt": {
        "as_u64": 65544,
        "as_i64": 32772
      }
    }
  },
…
```
