# VoteOS Deployment Checklist

> Complete every item before declaring the system ready for pilot use.

---

## Pre-Deployment

- [ ] Config file created from template (`config/templates/voteos.pilot.toml`)
- [ ] API key set to a strong, unique value (minimum 16 characters recommended)
- [ ] `require_auth = true`
- [ ] `persistence.enabled = true`
- [ ] `persistence.data_dir` points to a writable directory
- [ ] Binary built: `cargo build --release` (or Docker image built)

## Startup

- [ ] Server starts without FATAL errors
- [ ] Startup log shows: `VoteOS v0.1.0 starting`
- [ ] Startup log shows: `Persistence enabled, data_dir: ...`
- [ ] Startup log shows: `VoteOS ready on http://...`

## Health Verification

- [ ] `GET /health` returns `{"status":"ok"}`
- [ ] `GET /ready` returns `{"ready":true, ...}`
- [ ] `GET /status` returns module listing and stats

## Functional Verification

- [ ] Create election via API succeeds
- [ ] Election lifecycle: create → publish → open → close works
- [ ] Tally computation succeeds on closed election
- [ ] Certification succeeds on computed tally
- [ ] Audit verification returns `{"matches":true}`
- [ ] Export returns certified result bundle

## Auth Verification

- [ ] Request without API key returns 401
- [ ] Request with wrong API key returns 401
- [ ] Request with correct API key succeeds
- [ ] Health/ready/status endpoints work WITHOUT auth

## Persistence Verification

- [ ] JSON files appear in data directory after creating data
- [ ] Restart system
- [ ] Previously created elections still visible after restart
- [ ] Audit verification still passes after restart

## Operations Verification

- [ ] Pause/resume election works
- [ ] Incident flag/resolve works
- [ ] Paused election cannot be opened

## Final

- [ ] All above items checked
- [ ] No WARN or ERROR logs during verification
- [ ] System ready for pilot data import
