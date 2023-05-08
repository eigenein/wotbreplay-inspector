# `wotbreplay-inspector`

World of Tanks Blitz replay inspector in Rust. Uses [`eigenein/wotbreplay-parser`](https://github.com/eigenein/wotbreplay-parser) under the hood.

[![Crates.io](https://img.shields.io/crates/v/wotbreplay-inspector)](https://crates.io/crates/wotbreplay-inspector)
[![Last commit](https://img.shields.io/github/last-commit/eigenein/wotbreplay-inspector)](https://github.com/eigenein/wotbreplay-inspector/commits/main)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/eigenein/wotbreplay-inspector/.github/workflows/check.yaml)](https://github.com/eigenein/wotbreplay-inspector/actions)
![License: MIT](https://img.shields.io/crates/l/wotbreplay-inspector)

## Quickstart

## Inspecting battle results

### Convert known fields into JSON

```shell
wotbreplay-inspector battle-results 20221205_1409__zeekrab_A140_ASTRON_REX_105_2308651318200102307.wotbreplay
```

Example output:

```json5
{
  "timestamp": 1670245795,
  "players": [
    {
      "account_id": 534505602,
      "info": {
        "nickname": "Roberto_Cadenas_Diaz",
        "platoon_id": null,
        "team_number": 2,
        "clan_tag": "ORUGA",
// ...
```

**Note**: this ignores any unknown fields.

### Dump full decoded structure into JSON

Useful for manual inspection:

```shell
wotbreplay-inspector battle-results 20221205_1409__zeekrab_A140_ASTRON_REX_105_2308651318200102307.wotbreplay --raw
```

## Inspecting replay packets

```shell
wotbreplay-inspector dump-data 20230508_1237__helaas_pindakaas_A140_ASTRON_REX_105_2318392312630661347.wotbreplay
```

Example output:

```json lines
{"clock":0.0,"payload":{"EntityMethod":{"UpdateArena":{"field_number":17,"arguments":{"players":null}}}},"raw_payload":"e4064c002f000000020000001100"}
{"clock":0.0,"payload":{"Unknown":{"packet_type":35}},"raw_payload":"5f"}
{"clock":0.0,"payload":{"Unknown":{"packet_type":36}},"raw_payload":"5ff90200"}
```
