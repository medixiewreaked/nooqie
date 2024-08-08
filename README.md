Nooqie
---

### Introduction
*"They said it couldn't be done but we can rebuild him stronger, faster, uncut."*

Nooqie is a Discord bot with basic LLM functionality though the Ollama API

### Getting Started
**Environment variables**:
```bash
export DISCORD_TOKEN="your token here"
export NOOQIE_PREFIX="!" # <optional>
export OLLAMA_POST_URL="http://your.url/api/generate" # <optional>
export OLLAMA_MODEL="llama2-uncensored" # <optional>
export RUST_LOG=none,nooqie=info # <optional>
```
*Ollama [setup](https://github.com/ollama/ollama)*

```bash
cd ./nooqie
cargo run
```

![example](https://github.com/medixiewreaked/nooqie/blob/main/tapes/example.gif?raw=true)

### Example:
```
!llm Who is Berry McCaulkiner, and why is he contacting my wife?
```
