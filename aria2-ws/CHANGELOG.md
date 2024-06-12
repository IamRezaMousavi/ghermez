# Change Log

## 0.3.0

- Fix struct publicity for response structs
- Update `tokio-tungstenite` to `0.17`
- Methods that require GID are now using `&str`

## 0.4.0

- Refactor code to use channels instead of mutexes
- Remove default timeout
- Rename `Hooks` to `Callbacks`

## 0.5.0

Dependency updates:

```toml
tokio-tungstenite = "0.21"
base64 = "0.22"
snafu = "0.8"
serde_with = { version = "3", features = ["chrono"] }
```

- Improve callback execution to address possible execution miss.
- Fix `announce_list` type <https://github.com/ComfyFluffy/aria2-ws-rs/pull/3>.
