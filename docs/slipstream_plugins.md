# Slipstream Plugins

Slipstream lets an operator stream canonical mapping updates, staking rewards, and committed
blocks from a snarkOS client or validator. The node loads plugins from shared libraries. No
compile-time feature is required.

Plugins subscribe to:

- **Mapping updates** — each key/value write during canonical finalize.
- **Staking rewards** — one event per current staker during canonical finalize.
- **Blocks** — the little-endian encoding of a block, after that block is committed.

Prover nodes do not finalize blocks, so they do not accept `--slipstream-config`.

---

## Building a Plugin

Depend on `snarkvm-slipstream-plugin-interface` and implement `SlipstreamPlugin`. Compile the
crate as a `cdylib` and export `_create_plugin`:

```rust
#[no_mangle]
pub extern "C" fn _create_plugin() -> *mut dyn SlipstreamPlugin {
    Box::into_raw(Box::new(MyPlugin::new()))
}
```

---

## Plugin Config File (JSON5)

`libpath` is required. A relative path resolves from the config file's directory.

```json5
{
  libpath: "./libmy_plugin.so",
  name: "my_plugin",
}
```

---

## Starting a Node

Pass one or more `--slipstream-config` flags. The node loads those plugins and turns the stream
on. If a plugin fails to load, the node exits.

```bash
snarkos start --client \
  --slipstream-config ~/.aleo/plugins/my_plugin.json5

snarkos start --validator \
  --slipstream-config ~/.aleo/plugins/my_plugin.json5 \
  --slipstream-config ~/.aleo/plugins/metrics.json5
```

---

## Runtime Management via REST

These routes require JWT authentication. Start the node with `--nojwt` to disable it.

The routes return **503** when the node was started without `--slipstream-config`.

### List loaded plugins

```
GET /{network}/slipstream/plugins
```

### Load a plugin

```
POST /{network}/slipstream/plugins
Content-Type: application/json

{ "config_file": "/path/to/plugin.json5" }
```

Returns **422** when a plugin with that name is already loaded.

### Unload a plugin

```
DELETE /{network}/slipstream/plugins/{name}
```

Returns **404** when no plugin with that name is loaded.

`PUT` reload is not available. Unload the plugin and load it again, or restart the node.

---

## Notes

- A plugin error is logged and does not reject the block.
- A serialization failure skips that event.
- Shutdown calls `on_unload` on every loaded plugin.
