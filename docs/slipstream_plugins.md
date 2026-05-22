# Slipstream Plugins

Slipstream is a plugin system that lets operators stream canonical mapping updates and staking
rewards from snarkOS nodes to external services (databases, metrics pipelines, etc.) in
real time, without modifying node code.

---

## Overview

Slipstream plugins are statically linked into the `snarkos` binary at compile time. Each plugin
implements the `SlipstreamPlugin` trait from `snarkvm-slipstream-plugin-interface` and
self-registers via `inventory::submit!` at link time. The plugin manager inside `snarkVM`'s
`FinalizeStore` calls plugin hooks every time canonical finalize runs.

Plugins can subscribe to:

- **Mapping updates** — every key/value write that occurs during canonical finalize.
- **Staking rewards** — per-staker reward notifications.

Only **Validator** and **Client** nodes finalize blocks and therefore support plugins.
Prover nodes do not.

---

## Building a Plugin

### 1. Implement the trait

Add `snarkvm-slipstream-plugin-interface` and `inventory` as dependencies:

```toml
# Cargo.toml
[dependencies]
snarkvm-slipstream-plugin-interface = { git = "https://github.com/ProvableHQ/snarkVM.git" }
inventory = "0.3"
```

Implement `SlipstreamPlugin` and register with `inventory::submit!`:

```rust
use snarkvm_slipstream_plugin_interface::slipstream_plugin_interface::{
    SlipstreamPlugin, PluginRegistration,
};

struct MyPlugin;

impl SlipstreamPlugin for MyPlugin {
    fn name(&self) -> &'static str { "my-plugin" }
    // override on_load, on_broadcast, on_unload as needed
}

inventory::submit! {
    PluginRegistration {
        name: "my-plugin",
        factory: || Box::new(MyPlugin::new()),
    }
}
```

### 2. Add the plugin to snarkOS

Add your crate as an optional dependency in `snarkOS/node/Cargo.toml` under the
`slipstream-plugins` feature:

```toml
[dependencies.my-plugin]
path = "../../my-plugin"
optional = true

[features]
slipstream-plugins = [
    "snarkvm/slipstream-plugins",
    "dep:my-plugin",
    # ...
]
```

No other snarkOS or snarkVM code changes are required. snarkOS discovers all registered plugins
at startup via `inventory::iter::<PluginRegistration>()`.

See `slipstream-plugin-postgres` for a complete reference implementation.

---

## Plugin Config File (JSON5)

Each plugin is configured via a JSON5 file. The `name` field must match the name passed to
`inventory::submit!` in the plugin crate.

```json5
{
  // Required: must match the name registered via inventory::submit! in the plugin crate.
  name: "my-plugin",

  // Plugin-specific fields — read by the plugin's own on_load implementation.
  connection_string: "postgres://user:pass@localhost/aleo",
  batch_size: 100,
}
```

---

## Starting a Node with Plugins

Build snarkOS with the `slipstream-plugins` feature:

```bash
cargo build --release --features slipstream-plugins
```

Pass one or more config file paths at startup (comma-separated or repeated):

```bash
# Single plugin
snarkos start --nodetype validator \
  --slipstream-plugins ~/.aleo/plugins/postgres.json5

# Multiple plugins
snarkos start --nodetype validator \
  --slipstream-plugins ~/.aleo/plugins/postgres.json5,~/.aleo/plugins/metrics.json5
```

Plugins are initialized synchronously before the REST server and consensus engine start.
If any plugin's `on_load` returns an error, the node exits immediately — there is no
"start without the plugin" fallback.

---

## REST API

> **Authentication required.** All slipstream endpoints require a JWT bearer token.
> The token is printed to stdout at node startup and written to
> `<node_data_dir>/jwt_secret_<address>.txt`.

### List active plugins

```
GET /{network}/slipstream/plugins
```

Response (200):
```json
["postgres", "metrics"]
```

---

## Example: curl Commands

```bash
BASE="http://localhost:3030/mainnet"
TOKEN="<your-jwt-token>"

# List active plugins
curl -H "Authorization: Bearer $TOKEN" "$BASE/slipstream/plugins"
```

---

## Notes

- Plugins are initialized in the order config files are provided and unloaded in reverse on shutdown.
- `on_unload` is called for every plugin during graceful shutdown.
- Errors returned from `on_broadcast` are logged as warnings and never propagated to the node —
  a misbehaving plugin cannot affect consensus.
- To update a plugin's config, stop the node, update the config file, and restart.
