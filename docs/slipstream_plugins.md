# Slipstream Plugins

Slipstream is a plugin system that lets operators stream canonical mapping updates and staking
rewards from snarkOS nodes to external services (databases, metrics pipelines, etc.) in
real time, without modifying node code.

---

## Overview

Slipstream plugins are dynamically loaded shared libraries (`.so` / `.dylib` / `.dll`) that
implement the `SlipstreamPlugin` trait from `snarkvm-slipstream-plugin-interface`. The plugin
manager inside `snarkVM`'s `FinalizeStore` calls plugin hooks every time canonical finalize runs.

Plugins can subscribe to:

- **Mapping updates** — every key/value write that occurs during canonical finalize.
- **Staking rewards** — per-staker reward notifications

Only **Validator** and **Client** nodes finalize blocks and therefore support plugins.
Prover nodes do not.

---

## Building a Plugin

Use `snarkvm-slipstream-plugin-interface` as a dependency and implement the `SlipstreamPlugin`
trait. Compile your crate as a `cdylib`:

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
snarkvm-slipstream-plugin-interface = { git = "https://github.com/ProvableHQ/snarkVM.git", branch = "stream_plugin_testing" }
```

Export the constructor with the exact symbol name `_create_plugin`:

```rust
#[no_mangle]
pub extern "C" fn _create_plugin() -> *mut dyn SlipstreamPlugin {
    Box::into_raw(Box::new(MyPlugin::new()))
}
```

See `slipstream-plugin-postgres` in the snarkVM repository for a complete reference
implementation.

---

## Plugin Config File (JSON5)

Each plugin is configured via a JSON5 file. The required field is `libpath`, which can be
absolute or relative to the config file's directory.

```json5
{
  // Required: path to the compiled .so / .dylib
  libpath: "./libslipstream_postgres_example.so",

  // Optional: override the plugin name reported by name()
  name: "postgres",

  // Plugin-specific fields (passed verbatim to on_load)
  connection_string: "postgres://user:pass@localhost/aleo",
  batch_size: 100,
}
```

---

## Starting a Node with Plugins

Compile snarkOS with the `slipstream-plugins` feature

```bash
cargo build --features slipstream-plugins
```

Pass one or more `--slipstream-config` flags at startup:

```bash
# Single plugin
snarkos start --client \
  --slipstream-config ~/.aleo/plugins/postgres/plugin.json5

# Multiple plugins
snarkos start --validator \
  --slipstream-config ~/.aleo/plugins/postgres/plugin.json5 \
  --slipstream-config ~/.aleo/plugins/metrics/plugin.json5
```

Plugins are loaded synchronously before the REST server starts. If any plugin fails to load,
the node exits with an error.

---

## Runtime Management via REST API

> **Authentication required.** All slipstream management endpoints are protected by JWT
> authentication. Every request must include an `Authorization: Bearer <token>` header.
> The token is printed to stdout at node startup and written to
> `<node_data_dir>/jwt_secret_<address>.txt`. To disable auth entirely, start the node
> with `--nojwt` (not recommended in production).

### List loaded plugins

```
GET /{network}/slipstream/plugins
```

Response (200):
```json
["postgres", "metrics"]
```

### Load a plugin at runtime

```
POST /{network}/slipstream/plugins
Content-Type: application/json

{ "config_file": "/path/to/plugin.json5" }
```

Response (200):
```json
{ "loaded": "postgres" }
```

Returns **422 Unprocessable Entity** if a plugin with that name is already loaded.

### Unload a plugin

```
DELETE /{network}/slipstream/plugins/{name}
```

Response (200):
```json
{ "unloaded": true }
```

Returns **404 Not Found** if no plugin with that name is loaded.

### Reload a plugin (not yet implemented)

`PUT /{network}/slipstream/plugins/{name}` is not currently available. To update a plugin's
config during runtime, unload it with DELETE and reload it with POST. Otherwise, stop the snarkos service, update the config, and restart it, pointing at the new config.

---

## Example: curl Commands

```bash
BASE="http://localhost:3030/mainnet"
TOKEN="<your-jwt-token>"   # printed at startup or found in <data_dir>/jwt_secret_<address>.txt (e.g. `~/.aleo/storage/jwt_secrect_{address}.txt`)

# List
curl -H "Authorization: Bearer $TOKEN" "$BASE/slipstream/plugins"

# Load
curl -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"config_file":"/path/to/plugin.json5"}' \
  "$BASE/slipstream/plugins"

# Unload
curl -X DELETE -H "Authorization: Bearer $TOKEN" "$BASE/slipstream/plugins/postgres"
```

---

## Notes

- Plugins are loaded in startup order and unloaded in reverse order on shutdown.
- The `on_unload` method is called on every plugin during graceful shutdown.
- Plugin errors during `notify_mapping_update` / `notify_staking_reward` are logged as warnings
  and never propagated to the node — a misbehaving plugin cannot crash the node.
- The plugin manager uses a `std::sync::RwLock`; `notify_*` calls acquire a read lock, while
  load/unload/reload operations acquire the write lock. Avoid long-running operations inside
  plugin callbacks.
