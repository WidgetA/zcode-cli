# zcode-cli

**zcode-cli** is a coding-agent CLI that runs locally on your computer. It is a fork of
[OpenAI Codex](https://github.com/openai/codex) (Apache-2.0) with the default backend switched to
[Zhipu GLM Coding Plan](https://open.bigmodel.cn) — no ChatGPT account required.

zcode-cli 是一个本地运行的编程智能体 CLI，基于 OpenAI Codex 分叉而来，默认接入智谱 GLM Coding Plan，
无需 ChatGPT 账号，配置 API Key 即可使用。

---

## Quickstart

### Install

Build and install from source with Cargo:

```shell
cargo install --path codex-rs/cli
```

This installs the `zcode` binary. Prebuilt binaries are also available from the
[GitHub Releases](https://github.com/WidgetA/zcode-cli/releases) page.

也可以直接从 [GitHub Releases](https://github.com/WidgetA/zcode-cli/releases) 下载对应平台的预编译二进制。

### Configure your API key

Get an API key from the [GLM Coding Plan](https://open.bigmodel.cn). Easiest: run `zcode login`
and paste the key when prompted — it is stored in `~/.zcode/auth.json` and picked up from then on:

```shell
zcode login
```

Alternatively set it in your environment:

```shell
# macOS / Linux
export ZHIPU_API_KEY="your-api-key"

# Windows (PowerShell)
$env:ZHIPU_API_KEY = "your-api-key"
```

`ZCODE_API_KEY` is also accepted as an alias when `ZHIPU_API_KEY` is not set. Environment variables
take precedence over a stored key. `zcode logout` removes the stored key.
（也可以直接运行 `zcode login` 粘贴 Key 保存；未设置 `ZHIPU_API_KEY` 时可用 `ZCODE_API_KEY` 别名；
环境变量优先于已保存的 Key。）

### Run

```shell
zcode
```

On first run, zcode asks you to confirm trust for the current directory, then drops you into the
interactive TUI. For non-interactive use, try `zcode exec "fix the failing tests"`.

## Configuration

Configuration lives in `~/.zcode/config.toml` (an existing `~/.codex` directory from upstream Codex
is still respected, and the `CODEX_HOME` environment variable works as before; `ZCODE_HOME` takes
priority if set).

The default model is `glm-5.3` on the built-in `glm` provider. To switch models:

```shell
zcode --model glm-5.2
```

or set it permanently in `~/.zcode/config.toml`:

```toml
model = "glm-5.2"
```

To use a different provider entirely, point `model_provider` at any entry under
`[model_providers]` (see `docs/config.md` for the full schema):

```toml
model_provider = "glm"

[model_providers.glm]
name = "GLM (Zhipu)"
base_url = "https://open.bigmodel.cn/api/v1"
env_key = "ZHIPU_API_KEY"
wire_api = "responses"
```

## Updating

If you installed from a prebuilt release, `zcode update` checks
[this repository's releases](https://github.com/WidgetA/zcode-cli/releases) and updates in place
where possible. Otherwise, re-run the `cargo install --path codex-rs/cli` command above.

## Privacy

Analytics (including the upstream Statsig metrics exporter) and feedback uploads are **disabled by
default** in this fork. You can opt in via `[analytics] enabled = true` / `[feedback] enabled = true`
in `config.toml`, but note these targets are upstream OpenAI endpoints.

## Attribution & license

zcode-cli is a fork of [openai/codex](https://github.com/openai/codex), licensed under
[Apache-2.0](LICENSE). All credit for the upstream project goes to OpenAI and the Codex
contributors; see `NOTICE` for details.

## Disclaimer

This project is an independent community fork. It is **not affiliated with, endorsed by, or
sponsored by Zhipu (智谱)**, and it is **not** Zhipu's official "ZCode" product. It is also not
affiliated with OpenAI. "GLM" and "Zhipu" are trademarks of their respective owners.
