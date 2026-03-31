# VoteOS Quickstart

> From zero to running election in under 10 minutes.

---

## 1. Build (2 minutes)

```bash
git clone https://github.com/WoodrowLove/voteOS.git
cd voteOS
cargo build --release
```

## 2. Configure (1 minute)

```bash
cp config/templates/voteos.dev.toml config/voteos.toml
```

For development, the defaults work out of the box. No changes needed.

## 3. Start (10 seconds)

```bash
./target/release/voteos config/voteos.toml
```

You should see:
```
INFO voteos: VoteOS v0.1.0 starting
INFO voteos: VoteOS ready on http://127.0.0.1:3100
```

## 4. Verify (30 seconds)

```bash
curl http://localhost:3100/health
# {"status":"ok","system":"voteos","version":"0.1.0"}
```

## 5. Run Your First Election (5 minutes)

```bash
API="http://localhost:3100"

# Create election
curl -s -X POST $API/api/elections/create \
  -H "Content-Type: application/json" \
  -d '{"title":"My First Election","description":"Testing VoteOS","election_type":"General","scope":"test"}'

# Copy the election_ref from the response, then:
ELEC="elec-1-20260331..."  # paste your actual ref here

# Publish → Open → Close
curl -s -X POST $API/api/elections/$ELEC/publish
curl -s -X POST $API/api/elections/$ELEC/open
curl -s -X POST $API/api/elections/$ELEC/close

# Compute tally
curl -s -X POST $API/api/tally/$ELEC/compute

# Certify result
curl -s -X POST $API/api/certify/$ELEC

# Verify audit
curl -s -X POST $API/api/audit/$ELEC/verify

# Export result
curl -s $API/api/export/$ELEC
```

## Done

You've just created, run, certified, audited, and exported an election result.

For the full operator guide, see [docs/runbooks/VOTEOS_OPERATOR_RUNBOOK.md](runbooks/VOTEOS_OPERATOR_RUNBOOK.md).
