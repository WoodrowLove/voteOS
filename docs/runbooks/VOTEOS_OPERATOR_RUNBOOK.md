# VoteOS Operator Runbook

> For operators deploying and running VoteOS.
> You do NOT need to read source code to use this system.

---

## A. Installation

### Option 1: Local Build (Requires Rust)

```bash
# 1. Clone the repository
git clone https://github.com/WoodrowLove/voteOS.git
cd voteOS

# 2. Ensure AxiaSystem-Rust-Bridge is available (sibling directory)
# ls ../AxiaSystem-Rust-Bridge/Cargo.toml

# 3. Build
cargo build --release

# 4. Binary is at: target/release/voteos
```

### Option 2: Docker

```bash
# Build image
docker build -t voteos .

# Run with persistent data
docker run -d \
  --name voteos \
  -p 3100:3100 \
  -v voteos-data:/data \
  voteos
```

---

## B. Configuration

Copy a template and customize:

```bash
# For development
cp config/templates/voteos.dev.toml config/voteos.toml

# For pilot deployment
cp config/templates/voteos.pilot.toml config/voteos.toml
```

### Required settings for pilot:

| Setting | Description | Example |
|---------|-------------|---------|
| `api.bind_address` | Network interface | `0.0.0.0` for network, `127.0.0.1` for local |
| `api.bind_port` | HTTP port | `3100` |
| `security.api_key` | Auth key (min 8 chars) | `your-secure-key-here` |
| `security.require_auth` | Enable auth | `true` for pilot |
| `persistence.data_dir` | Data storage path | `/data` or `./data` |
| `persistence.enabled` | Enable disk storage | `true` (required for pilot) |

---

## C. Starting the System

```bash
# Local
./target/release/voteos config/voteos.toml

# Or with cargo
cargo run --release -- config/voteos.toml
```

### Expected startup logs:

```
INFO voteos: VoteOS v0.1.0 starting
INFO voteos: Persistence enabled, data_dir: /data
INFO voteos: Registries initialized — elections: 0, certifications: 0, auth: enabled
INFO voteos: VoteOS ready on http://0.0.0.0:3100
```

### Startup will FAIL if:
- Config file missing or unparseable
- `require_auth=true` with empty or short API key
- Persistence enabled but data directory not writable
- Bind address/port invalid or already in use

---

## D. Verification

After startup, verify the system is healthy:

```bash
# Health check (no auth required)
curl http://localhost:3100/health

# Readiness check (no auth required)
curl http://localhost:3100/ready

# System status (no auth required)
curl http://localhost:3100/status
```

### Run a test election lifecycle:

```bash
API="http://localhost:3100"
KEY="your-api-key"
AUTH="-H x-api-key:$KEY"

# 1. Create election
curl -s -X POST $API/api/elections/create $AUTH \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Election","description":"Verification test","election_type":"General","scope":"test"}'

# Response: {"success":true,"data":{"election_ref":"elec-1-..."}}
# Save the election_ref for next steps

ELEC="elec-1-20260331..."  # Use the actual ref from response

# 2. Publish
curl -s -X POST $API/api/elections/$ELEC/publish $AUTH

# 3. Open
curl -s -X POST $API/api/elections/$ELEC/open $AUTH

# 4. Close
curl -s -X POST $API/api/elections/$ELEC/close $AUTH

# 5. Compute tally
curl -s -X POST $API/api/tally/$ELEC/compute $AUTH

# 6. Certify
curl -s -X POST $API/api/certify/$ELEC $AUTH

# 7. Verify audit
curl -s -X POST $API/api/audit/$ELEC/verify $AUTH

# 8. Export result
curl -s $API/api/export/$ELEC $AUTH
```

---

## E. Common Operations

### Create an election
```bash
curl -X POST $API/api/elections/create $AUTH \
  -H "Content-Type: application/json" \
  -d '{"title":"2026 City Council","description":"General election","election_type":"General","scope":"city"}'
```

### List all elections
```bash
curl $API/api/elections $AUTH
```

### Get election details
```bash
curl $API/api/elections/$ELEC $AUTH
```

### Lifecycle transitions
```bash
curl -X POST $API/api/elections/$ELEC/publish $AUTH
curl -X POST $API/api/elections/$ELEC/open $AUTH
curl -X POST $API/api/elections/$ELEC/close $AUTH
```

### Compute tally
```bash
curl -X POST $API/api/tally/$ELEC/compute $AUTH
```

### Certify result
```bash
curl -X POST $API/api/certify/$ELEC $AUTH
```

### View tally
```bash
curl $API/api/tally/$ELEC $AUTH
```

### Audit election
```bash
curl $API/api/audit/$ELEC $AUTH          # View bundle summary
curl -X POST $API/api/audit/$ELEC/verify $AUTH  # Run verification
```

### Export certified result
```bash
curl $API/api/export/$ELEC $AUTH
```

### Operational controls
```bash
# Pause election
curl -X POST $API/api/operations/$ELEC/pause $AUTH \
  -H "Content-Type: application/json" \
  -d '{"reason":"Weather emergency"}'

# Resume
curl -X POST $API/api/operations/$ELEC/resume $AUTH \
  -H "Content-Type: application/json" \
  -d '{"reason":"Emergency resolved"}'

# Flag incident
curl -X POST $API/api/operations/$ELEC/incident $AUTH \
  -H "Content-Type: application/json" \
  -d '{"reason":"Machine malfunction at precinct 7"}'

# View operational state
curl $API/api/operations/$ELEC/state $AUTH
```

---

## F. Troubleshooting

### System won't start

| Symptom | Cause | Fix |
|---------|-------|-----|
| `FATAL: Failed to read config` | Config file missing | Check path: `./target/release/voteos config/voteos.toml` |
| `FATAL: Failed to parse config` | TOML syntax error | Validate config with `cat config/voteos.toml` |
| `FATAL: require_auth=true but api_key is empty` | Auth misconfigured | Set a key in config or set `require_auth = false` |
| `FATAL: api_key must be at least 8 characters` | Key too short | Use a longer key |
| `FATAL: Failed to bind` | Port in use | Change `bind_port` or stop conflicting service |
| `FATAL: Data directory not writable` | Permission issue | Check permissions: `chmod 755 /data` |

### API returns 401 Unauthorized

- Check `x-api-key` header matches config `api_key` exactly
- Ensure header format: `-H "x-api-key: your-key-here"`
- Health/ready/status endpoints do NOT require auth

### API returns 400 Bad Request

- Check the error message in the response `"error"` field
- Common: wrong election status for requested operation
- Common: election not found (check election_ref)

### Data not persisting after restart

- Verify `persistence.enabled = true` in config
- Verify `data_dir` path exists and is writable
- Check for `.json` files in the data directory

---

## G. Log Levels

| Level | Meaning |
|-------|---------|
| `INFO` | Normal operations — startup, requests served |
| `WARN` | Non-fatal issues — persistence disabled, auto-save failure |
| `ERROR` | Failures — request handling errors |
| `FATAL` | Startup failures — system cannot run |

Set log level via environment:
```bash
RUST_LOG=voteos=debug ./target/release/voteos config/voteos.toml
```
