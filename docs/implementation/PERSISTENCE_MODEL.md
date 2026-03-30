# CivilOS Persistence Model

> V1 file-backed persistence for domain registries.
> Eliminates the highest operational risk: in-memory state loss on restart.

---

## Storage Layout

```
data/
  identity_admin.json    # DepartmentRegistry (departments + role assignments)
  dmv.json               # DmvRegistry (licenses, vehicles, titles)
  permits.json           # PermitRegistry (applications, permits, violations)
  finance.json           # FinanceRegistry (transactions, budgets, treasury)
  public_safety.json     # PublicSafetyRegistry (incidents, cases, citations)
  assets.json            # AssetsRegistry (records + history)
  governance.json        # GovernanceRegistry (proposals, approvals, decisions)
  citizen_services.json  # CitizenServicesRegistry (requests, audit log)
```

Each file is a JSON serialization of a `StoreSnapshot` containing:
- `prefix`: store identifier
- `counter`: ID generation counter (ensures unique IDs after reload)
- `records`: HashMap of all stored entities

---

## What Is Persisted

Every `DomainStore<T>` instance that is created with `DomainStore::with_persistence()` persists:
- All records in the store (keyed by string ID)
- The ID generation counter (so new IDs don't collide after restart)
- The store prefix

## What Is NOT Persisted

- Workflow execution state (in-flight operations)
- SpineClient connections (re-established on startup)
- AxiaSystem canister state (lives on ICP, not in CivilOS files)
- Wrapper mapping state (separate persistence if needed)

---

## When Saves Occur

- **`insert_new()`**: auto-saves after every new record creation
- **`update()`**: auto-saves after every record update
- **`save()`**: explicit save (can be called by application code at any point)

Failed operations do NOT trigger saves — only successful mutations.

---

## Load Behavior

On `DomainStore::with_persistence()`:
1. If file exists and is valid JSON → load records + counter
2. If file does not exist → start with empty state (normal for first run)
3. If file is corrupt → start with empty state, log warning to stderr

---

## Corrupt/Missing File Handling

| Condition | Behavior | Recovery |
|-----------|----------|----------|
| File missing | Clean start (empty state) | Normal first-run behavior |
| File empty | Error returned by `load_snapshot` | `with_persistence` starts empty |
| File corrupt JSON | Error returned by `load_snapshot` | `with_persistence` starts empty, warning logged |
| Write failure | Warning logged to stderr | In-memory state remains authoritative |

**Save uses atomic write:** writes to `.tmp` file first, then renames. This prevents partial writes from corrupting the persisted state.

---

## How to Enable Persistence

Replace `DomainStore::new("prefix")` with:

```rust
DomainStore::with_persistence("prefix", PathBuf::from("data/module_name.json"))
```

The existing `DomainStore::new()` API is unchanged — non-persistent stores continue to work identically.

---

## Limitations (v1)

1. **Save-per-mutation.** Every insert/update writes the entire store to disk. For high-volume modules this may become a bottleneck. v2 should implement write-ahead logging or incremental persistence.
2. **No encryption.** Persisted JSON is plaintext. Sensitive data (financial transactions, citizen records) is stored in the clear.
3. **No backup/rotation.** Only one file per store. No automatic backups or log rotation.
4. **Atomic write is best-effort.** The rename-after-write pattern is atomic on most filesystems but not guaranteed on all (e.g., network-mounted drives).
5. **Wrapper state not yet persisted.** Identity Reconciler mapping store, adapter state, and cutover controller state remain in-memory.

---

## Path to v2 (Database-Backed)

The persistence model is designed for future migration:
- `StoreSnapshot` can be replaced with database table reads/writes
- `save()` → SQL INSERT/UPDATE
- `load()` → SQL SELECT
- The `DomainStore` API surface stays the same
- Alternatively, migrate domain state to a CivilOS ICP canister for on-chain durability
