# `wotbreplay-inspector`

World of Tanks Blitz replay inspector in Rust. Uses [`eigenein/wotbreplay-parser`](https://github.com/eigenein/wotbreplay-parser) under the hood.

[![Crates.io](https://img.shields.io/crates/v/wotbreplay-inspector)](https://crates.io/crates/wotbreplay-inspector)
[![Last commit](https://img.shields.io/github/last-commit/eigenein/wotbreplay-inspector)](https://github.com/eigenein/wotbreplay-inspector/commits/main)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/eigenein/wotbreplay-inspector/Check)](https://github.com/eigenein/wotbreplay-inspector/actions)
![License: MIT](https://img.shields.io/crates/l/wotbreplay-inspector)

## Quickstart

### Convert known fields into JSON

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

Note: this ignores any unknown fields.

### Dump full decoded structure into [TOML](https://toml.io)

Useful for manual inspection:

```
❯ wotbreplay-inspector 20221205_1409__zeekrab_A140_ASTRON_REX_105_2308651318200102307.wotbreplay battle-results --raw
# 1: varint
1 = { u64 = 65543, i64 = -32772 }
# 2: varint
2 = { u64 = 1670282196, i64 = 835141098 }
# 3: varint
3 = { u64 = 1, i64 = -1 }
# 4: varint
4 = { u64 = 1, i64 = -1 }
# 5: varint
5 = { u64 = 345, i64 = -173 }

# start message #8
[8]
# 2: varint
2 = { u64 = 32250, i64 = 16125 }
…
```

Tip: it's supposed to be `diff`-friendly to compare fields between different replays.
