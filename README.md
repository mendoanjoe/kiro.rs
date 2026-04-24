# kiro-rs

An Anthropic Claude API-compatible proxy service written in Rust that converts Anthropic API requests into Kiro API requests.

---

<table>
<tr>
<td>
<b>Special Thanks</b>: <a href="https://co.yes.vg/register?ref=hank9999">YesCode</a> for sponsoring AI API credits for this project. YesCode is a low-key, pragmatic AI API relay service provider <br>
that has long delivered stable and highly available service. If you are interested in trying it out, click the link to register → <a href="https://co.yes.vg/register?ref=hank9999">Visit Now</a>
</td>
</tr>
</table>

---

#### [LINUX DO Discussion Thread](https://linux.do/t/topic/1571986)

## Disclaimer

This project is for research purposes only. Use at your own risk. Any consequences resulting from the use of this project are solely the responsibility of the user and are unrelated to this project.
This project is not affiliated with AWS/KIRO/Anthropic/Claude or any other official entity and does not represent any official position.

## Important!

Since TLS has been switched from native-tls to rustls by default, you may need to install certificates explicitly before configuring an HTTP proxy. You can switch back to `native-tls` via the `tlsBackend` field in `config.json`.
If you encounter request errors — especially failures to refresh the token or direct `error request` responses — try switching the TLS backend to `native-tls`, which usually resolves the issue.

**Write Failed / Session Stuck**: If you encounter persistent Write File / Write Failed errors that render a session unusable, refer to the notes and temporary workarounds in Issues [#22](https://github.com/hank9999/kiro.rs/issues/22) and [#49](https://github.com/hank9999/kiro.rs/issues/49) (usually related to output being truncated due to excessive length — try lowering the output token limit).

## Features

- **Anthropic API Compatible**: Full support for the Anthropic Claude API format
- **Streaming Responses**: Supports SSE (Server-Sent Events) streaming output
- **Automatic Token Refresh**: Automatically manages and refreshes OAuth tokens
- **Multiple Credentials**: Supports configuring multiple credentials with automatic priority-based failover
- **Load Balancing**: Supports `priority` (by priority order) and `balanced` (round-robin) modes
- **Smart Retry**: Up to 3 retries per credential, up to 9 retries per request
- **Credential Writeback**: Automatically writes refreshed tokens back to the source file in multi-credential mode
- **Thinking Mode**: Supports Claude's extended thinking feature
- **Tool Use**: Full support for function calling / tool use
- **WebSearch**: Built-in WebSearch tool conversion logic
- **Multi-Model Support**: Supports Sonnet, Opus, and Haiku model families
- **Admin Panel**: Optional web management interface and API supporting credential management, balance queries, and more
- **Multi-Level Region Configuration**: Supports global and credential-level Auth Region / API Region configuration
- **Per-Credential Proxy**: Supports configuring an HTTP/SOCKS5 proxy per credential; priority: credential proxy > global proxy > no proxy

---

- [Getting Started](#getting-started)
  - [1. Build](#1-build)
  - [2. Minimal Configuration](#2-minimal-configuration)
  - [3. Start](#3-start)
  - [4. Verify](#4-verify)
  - [Docker](#docker)
- [Configuration Reference](#configuration-reference)
  - [config.json](#configjson)
  - [credentials.json](#credentialsjson)
  - [Region Configuration](#region-configuration)
  - [Proxy Configuration](#proxy-configuration)
  - [Authentication Methods](#authentication-methods)
  - [Environment Variables](#environment-variables)
- [API Endpoints](#api-endpoints)
  - [Standard Endpoints (/v1)](#standard-endpoints-v1)
  - [Claude Code Compatible Endpoints (/cc/v1)](#claude-code-compatible-endpoints-ccv1)
  - [Thinking Mode](#thinking-mode)
  - [Tool Use](#tool-use)
- [Model Mapping](#model-mapping)
- [Admin (Optional)](#admin-optional)
- [Notes](#notes)
- [Project Structure](#project-structure)
- [Tech Stack](#tech-stack)
- [License](#license)
- [Acknowledgements](#acknowledgements)

## Getting Started

### 1. Build

> PS: If you don't want to build from source, you can download a pre-built binary from the Releases page.

> **Prerequisites**: Before building, you must first build the frontend Admin UI (to be embedded in the binary):
> ```bash
> cd admin-ui && pnpm install && pnpm build
> ```

```bash
cargo build --release
```

### 2. Minimal Configuration

Create `config.json`:

```json
{
   "host": "127.0.0.1",
   "port": 8990,
   "apiKey": "sk-kiro-rs-qazWSXedcRFV123456",
   "region": "us-east-1"
}
```
> PS: If you need the web admin panel, make sure to configure `adminApiKey`.

Create `credentials.json` (obtain credential information from the Kiro IDE or similar):
> PS: You can skip this step by configuring credentials via the web admin panel.
> If you are unsure about credential regions, see [Region Configuration](#region-configuration).

Social authentication:
```json
{
   "refreshToken": "your-refresh-token",
   "expiresAt": "2025-12-31T02:32:45.144Z",
   "authMethod": "social"
}
```

IdC authentication:
```json
{
   "refreshToken": "your-refresh-token",
   "expiresAt": "2025-12-31T02:32:45.144Z",
   "authMethod": "idc",
   "clientId": "your-client-id",
   "clientSecret": "your-client-secret"
}
```

### 3. Start

```bash
./target/release/kiro-rs
```

Or specify config file paths explicitly:

```bash
./target/release/kiro-rs -c /path/to/config.json --credentials /path/to/credentials.json
```

### 4. Verify

```bash
curl http://127.0.0.1:8990/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-kiro-rs-qazWSXedcRFV123456" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "max_tokens": 1024,
    "stream": true,
    "messages": [
      {"role": "user", "content": "Hello, Claude!"}
    ]
  }'
```

### Docker

You can also start the service via Docker:

```bash
docker-compose up
```

Mount `config.json` and `credentials.json` into the container as described in `docker-compose.yml`.

## Configuration Reference

### config.json

| Field | Type | Default | Description |
|------|------|--------|------|
| `host` | string | `127.0.0.1` | Service listen address |
| `port` | number | `8080` | Service listen port |
| `apiKey` | string | - | Custom API key (used for client authentication; required) |
| `region` | string | `us-east-1` | AWS region |
| `authRegion` | string | - | Auth region (for token refresh); falls back to `region` if not set |
| `apiRegion` | string | - | API region (for API requests); falls back to `region` if not set |
| `kiroVersion` | string | `0.9.2` | Kiro version string |
| `machineId` | string | - | Custom machine ID (64-character hex); auto-generated if not set |
| `systemVersion` | string | random | System version identifier |
| `nodeVersion` | string | `22.21.1` | Node.js version identifier |
| `tlsBackend` | string | `rustls` | TLS backend: `rustls` or `native-tls` |
| `countTokensApiUrl` | string | - | External count_tokens API URL |
| `countTokensApiKey` | string | - | External count_tokens API key |
| `countTokensAuthType` | string | `x-api-key` | External API auth type: `x-api-key` or `bearer` |
| `proxyUrl` | string | - | HTTP/SOCKS5 proxy URL |
| `proxyUsername` | string | - | Proxy username |
| `proxyPassword` | string | - | Proxy password |
| `adminApiKey` | string | - | Admin API key; enables the credential management API and web admin UI when set |
| `loadBalancingMode` | string | `priority` | Load balancing mode: `priority` (by priority order) or `balanced` (round-robin) |
| `extractThinking` | boolean | `true` | Thinking block extraction for non-streaming responses. When enabled, `<thinking>` tags are parsed into standalone `thinking` content blocks |
| `defaultEndpoint` | string | `ide` | Default Kiro endpoint. Used when a credential does not explicitly specify `endpoint`. Currently supported: `ide` |

Full configuration example:

```json
{
   "host": "127.0.0.1",
   "port": 8990,
   "apiKey": "sk-kiro-rs-qazWSXedcRFV123456",
   "region": "us-east-1",
   "tlsBackend": "rustls",
   "kiroVersion": "0.9.2",
   "machineId": "your-64-char-hex-machine-id",
   "systemVersion": "darwin#24.6.0",
   "nodeVersion": "22.21.1",
   "authRegion": "us-east-1",
   "apiRegion": "us-east-1",
   "countTokensApiUrl": "https://api.example.com/v1/messages/count_tokens",
   "countTokensApiKey": "sk-your-count-tokens-api-key",
   "countTokensAuthType": "x-api-key",
   "proxyUrl": "http://127.0.0.1:7890",
   "proxyUsername": "user",
   "proxyPassword": "pass",
   "adminApiKey": "sk-admin-your-secret-key",
   "loadBalancingMode": "priority",
   "extractThinking": true
}
```

### credentials.json

Supports a single-object format (backward compatible) or an array format (multiple credentials).

#### Field Reference

| Field          | Type   | Description                                                                                     |
|----------------|--------|-------------------------------------------------------------------------------------------------|
| `id`           | number | Unique credential ID (optional; only used for Admin API management; can be omitted in hand-written files) |
| `accessToken`  | string | OAuth access token (optional; can be auto-refreshed)                                            |
| `refreshToken` | string | OAuth refresh token                                                                             |
| `profileArn`   | string | AWS Profile ARN (optional; returned at login)                                                   |
| `expiresAt`    | string | Token expiry time (RFC3339)                                                                     |
| `authMethod`   | string | Authentication method: `social` or `idc`                                                        |
| `clientId`     | string | Client ID for IdC login (required for IdC authentication)                                       |
| `clientSecret` | string | Client secret for IdC login (required for IdC authentication)                                   |
| `priority`     | number | Credential priority; lower number means higher priority; default is 0                           |
| `region`       | string | Credential-level Auth Region; compatibility field                                               |
| `authRegion`   | string | Credential-level Auth Region for token refresh; falls back to `region` if not set              |
| `apiRegion`    | string | Credential-level API Region for API requests                                                    |
| `machineId`    | string | Credential-level machine ID (64-character hex)                                                  |
| `email`        | string | User email (optional; retrieved from API)                                                       |
| `proxyUrl`     | string | Credential-level proxy URL (optional; special value `direct` means no proxy)                   |
| `proxyUsername`| string | Credential-level proxy username (optional)                                                      |
| `proxyPassword`| string | Credential-level proxy password (optional)                                                      |
| `endpoint`     | string | Credential-level endpoint name (optional; uses `config.defaultEndpoint` if not set)            |

Notes:
- IdC / Builder-ID / IAM are all treated as the same login method in this project; use `authMethod: "idc"` for all of them.
- For backward compatibility, `builder-id` / `iam` are still recognized but will be handled as `idc`.

#### Single Credential Format (legacy format, backward compatible)

```json
{
   "accessToken": "access-token-valid-for-about-one-hour-optional",
   "refreshToken": "refresh-token-valid-for-7-to-30-days",
   "profileArn": "arn:aws:codewhisperer:us-east-1:111112222233:profile/QWER1QAZSDFGH",
   "expiresAt": "2025-12-31T02:32:45.144Z",
   "authMethod": "social",
   "clientId": "required-for-idc-login",
   "clientSecret": "required-for-idc-login"
}
```

#### Multiple Credentials Format (supports failover and automatic writeback)

```json
[
   {
      "refreshToken": "refresh-token-for-first-credential",
      "expiresAt": "2025-12-31T02:32:45.144Z",
      "authMethod": "social",
      "priority": 0
   },
   {
      "refreshToken": "refresh-token-for-second-credential",
      "expiresAt": "2025-12-31T02:32:45.144Z",
      "authMethod": "idc",
      "clientId": "xxxxxxxxx",
      "clientSecret": "xxxxxxxxx",
      "region": "us-east-2",
      "priority": 1,
      "proxyUrl": "socks5://proxy.example.com:1080",
      "proxyUsername": "user",
      "proxyPassword": "pass"
   },
   {
      "refreshToken": "refresh-token-for-third-credential-explicit-direct-connection",
      "expiresAt": "2025-12-31T02:32:45.144Z",
      "authMethod": "social",
      "priority": 2,
      "proxyUrl": "direct"
   }
]
```

Multiple credentials features:
- Sorted by the `priority` field; lower number means higher priority (default is 0)
- Up to 3 retries per credential, up to 9 retries per request
- Automatically fails over to the next available credential
- Refreshed tokens are automatically written back to the source file in multi-credential mode

### Region Configuration

Supports multi-level region configuration to independently control the region used for token refresh and API requests.

**Auth Region** (token refresh) priority:
`credential.authRegion` > `credential.region` > `config.authRegion` > `config.region`

**API Region** (API requests) priority:
`credential.apiRegion` > `config.apiRegion` > `config.region`

### Proxy Configuration

Supports both global and per-credential proxies. A credential-level proxy overrides all outbound connections for that credential (API requests, token refresh, balance queries).

**Proxy priority**: `credential.proxyUrl` > `config.proxyUrl` > no proxy

| Credential `proxyUrl` value | Behavior |
|---|---|
| A specific URL (e.g. `http://proxy:8080`, `socks5://proxy:1080`) | Uses the proxy specified by the credential |
| `direct` | Explicitly disables proxy (even if a global proxy is configured) |
| Not set (empty) | Falls back to the global proxy configuration |

Per-credential proxy example:

```json
[
   {
      "refreshToken": "credential-a-uses-its-own-proxy",
      "authMethod": "social",
      "proxyUrl": "socks5://proxy-a.example.com:1080",
      "proxyUsername": "user_a",
      "proxyPassword": "pass_a"
   },
   {
      "refreshToken": "credential-b-explicit-direct-connection-no-proxy",
      "authMethod": "social",
      "proxyUrl": "direct"
   },
   {
      "refreshToken": "credential-c-uses-global-proxy-or-direct-depending-on-config",
      "authMethod": "social"
   }
]
```

### Authentication Methods

Two authentication methods are supported for client requests to this service:

1. **x-api-key Header**
   ```
   x-api-key: sk-your-api-key
   ```

2. **Authorization Bearer**
   ```
   Authorization: Bearer sk-your-api-key
   ```

### Environment Variables

Log level can be configured via environment variable:

```bash
RUST_LOG=debug ./target/release/kiro-rs
```

## API Endpoints

### Standard Endpoints (/v1)

| Endpoint | Method | Description |
|------|------|------|
| `/v1/models` | GET | List available models |
| `/v1/messages` | POST | Create a message (chat) |
| `/v1/messages/count_tokens` | POST | Estimate token count |

### Claude Code Compatible Endpoints (/cc/v1)

| Endpoint | Method | Description |
|------|------|------|
| `/cc/v1/messages` | POST | Create a message (buffered mode, ensures accurate `input_tokens`) |
| `/cc/v1/messages/count_tokens` | POST | Estimate token count (same as `/v1`) |

> **Difference between `/cc/v1/messages` and `/v1/messages`**:
> - `/v1/messages`: Real-time streaming; `input_tokens` in `message_start` is an estimate.
> - `/cc/v1/messages`: Buffered mode; waits for the upstream stream to complete, then corrects `message_start` with the accurate `input_tokens` computed from `contextUsageEvent`, and returns all events at once.
> - A `ping` event is sent every 25 seconds during the wait to keep the connection alive.

### Thinking Mode

Supports Claude's extended thinking feature:

```json
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 16000,
  "thinking": {
    "type": "enabled",
    "budget_tokens": 10000
  },
  "messages": [...]
}
```

### Tool Use

Full support for Anthropic's tool use feature:

```json
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 1024,
  "tools": [
    {
      "name": "get_weather",
      "description": "Get the weather for a specified city",
      "input_schema": {
        "type": "object",
        "properties": {
          "city": {"type": "string"}
        },
        "required": ["city"]
      }
    }
  ],
  "messages": [...]
}
```

## Model Mapping

| Anthropic Model | Kiro Model |
|----------------|-----------|
| `*sonnet*` | `claude-sonnet-4.5` |
| `*opus*` (including 4.5/4-5) | `claude-opus-4.5` |
| `*opus*` (others) | `claude-opus-4.6` |
| `*haiku*` | `claude-haiku-4.5` |

## Admin (Optional)

When a non-empty `adminApiKey` is configured in `config.json`, the following are enabled:

- **Admin API (authenticated with the same API key)**
  - `GET /api/admin/credentials` - Get all credential statuses
  - `POST /api/admin/credentials` - Add a new credential
  - `DELETE /api/admin/credentials/:id` - Delete a credential
  - `POST /api/admin/credentials/:id/disabled` - Set credential disabled state
  - `POST /api/admin/credentials/:id/priority` - Set credential priority
  - `POST /api/admin/credentials/:id/reset` - Reset failure count
  - `GET /api/admin/credentials/:id/balance` - Get credential balance

- **Admin UI**
  - `GET /admin` - Access the admin page (requires `admin-ui/dist` to be built before compilation)

## Notes

1. **Credential Security**: Keep your `credentials.json` file safe and do not commit it to version control.
2. **Token Refresh**: The service automatically refreshes expired tokens; no manual intervention is needed.
3. **WebSearch Tool**: When the `tools` list contains only a single `web_search` tool, the built-in WebSearch conversion logic is used.

## Project Structure

```
kiro-rs/
├── src/
│   ├── main.rs                 # Program entry point
│   ├── http_client.rs          # HTTP client construction
│   ├── token.rs                # Token calculation module
│   ├── debug.rs                # Debug utilities
│   ├── test.rs                 # Tests
│   ├── model/                  # Configuration and parameter models
│   │   ├── config.rs           # Application configuration
│   │   └── arg.rs              # Command-line arguments
│   ├── anthropic/              # Anthropic API compatibility layer
│   │   ├── router.rs           # Route configuration
│   │   ├── handlers.rs         # Request handlers
│   │   ├── middleware.rs       # Authentication middleware
│   │   ├── types.rs            # Type definitions
│   │   ├── converter.rs        # Protocol converter
│   │   ├── stream.rs           # Streaming response handling
│   │   └── websearch.rs        # WebSearch tool handling
│   ├── kiro/                   # Kiro API client
│   │   ├── provider.rs         # API provider
│   │   ├── token_manager.rs    # Token management
│   │   ├── machine_id.rs       # Device fingerprint generation
│   │   ├── model/              # Data models
│   │   │   ├── credentials.rs  # OAuth credentials
│   │   │   ├── events/         # Response event types
│   │   │   ├── requests/       # Request types
│   │   │   ├── common/         # Shared types
│   │   │   ├── token_refresh.rs # Token refresh model
│   │   │   └── usage_limits.rs # Usage limits model
│   │   └── parser/             # AWS Event Stream parser
│   │       ├── decoder.rs      # Streaming decoder
│   │       ├── frame.rs        # Frame parsing
│   │       ├── header.rs       # Header parsing
│   │       ├── error.rs        # Error types
│   │       └── crc.rs          # CRC verification
│   ├── admin/                  # Admin API module
│   │   ├── router.rs           # Route configuration
│   │   ├── handlers.rs         # Request handlers
│   │   ├── service.rs          # Business logic service
│   │   ├── types.rs            # Type definitions
│   │   ├── middleware.rs       # Authentication middleware
│   │   └── error.rs            # Error handling
│   ├── admin_ui/               # Admin UI static file embedding
│   │   └── router.rs           # Static file routing
│   └── common/                 # Common modules
│       └── auth.rs             # Authentication utility functions
├── admin-ui/                   # Admin UI frontend project (build output is embedded in the binary)
├── tools/                      # Utility tools
├── Cargo.toml                  # Project configuration
├── config.example.json         # Configuration example
├── docker-compose.yml          # Docker Compose configuration
└── Dockerfile                  # Docker build file
```

## Tech Stack

- **Web Framework**: [Axum](https://github.com/tokio-rs/axum) 0.8
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **HTTP Client**: [Reqwest](https://github.com/seanmonstar/reqwest)
- **Serialization**: [Serde](https://serde.rs/)
- **Logging**: [tracing](https://github.com/tokio-rs/tracing)
- **CLI**: [Clap](https://github.com/clap-rs/clap)

## License

MIT

## Acknowledgements

This project stands on the shoulders of those who came before:
 - [kiro2api](https://github.com/caidaoli/kiro2api)
 - [proxycast](https://github.com/aiclientproxy/proxycast)

Parts of this project's logic were inspired by the above projects. Sincere thanks!
