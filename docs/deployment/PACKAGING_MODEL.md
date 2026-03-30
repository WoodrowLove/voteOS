# CivilOS Packaging Model

> Defines what constitutes a CivilOS deployment unit.

---

## Deployment Components

| Component | Location | Purpose |
|-----------|----------|---------|
| CivilOS binary | `target/release/civilos_server` | HTTP API server |
| City config | `config/city.*.toml` | Per-city settings |
| Persistence data | `deploy/cities/{slug}/data/` | Domain registry state |
| Scripts | `scripts/` | Install, bootstrap, verify |
| AxiaSystem | External (ICP canisters) | Core capabilities |
| Rust Bridge | External (../AxiaSystem-Rust-Bridge) | Bridge to canisters |

## Deployment Modes

| Mode | IC Host | Persistence | Wrapper |
|------|---------|-------------|---------|
| Local/Dev | 127.0.0.1:4943 | File-backed | Optional |
| Pilot | ICP mainnet | File-backed | Enabled |
| Production | ICP mainnet | File/DB-backed | Enabled → phased out |

## Directory Convention

```
deploy/
  cities/
    example_city/
      config/city.toml
      data/
        identity_admin.json
        dmv.json
        permits.json
        finance.json
        public_safety.json
        assets.json
        governance.json
        citizen_services.json
      logs/
      runtime/
```

## New City = New Config

A new city deployment requires:
1. Copy `config/city.template.toml` → `config/{city}.toml`
2. Fill in city-specific values (canister IDs, admin credentials, etc.)
3. Run `scripts/install_city.sh config/{city}.toml`
4. Run `scripts/bootstrap_city.sh` (configures AxiaSystem auth)
5. Run `scripts/verify_city_install.sh` (confirms operational)

No code changes. No repo forking.
