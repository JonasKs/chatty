# Terminal AI Ops

A proof of concept terminal emulator with an AI chat sidebar. Your terminal output is automatically injected into chat context, so you can ask questions about what's happening in your shell without copy/pasting.

Built in the summer of 2022, **before Claude Code or Codex CLI existed.**

[![demo](demo-2x-speed.gif)](https://share.klepp.me/?path=hotfix/ai-psuedo-terminal-2x-speed.mp4)
(Click the gif to watch it in video format)

## What it does

- Spawns a real pseudoterminal (PTY) inside a TUI
- AI chat panel runs alongside your terminal
- Terminal output is automatically captured and sent as context with your messages
- Streaming responses from GPT-4o
- Roles with custom prompts (such as `/network` and `/linux`)


## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for details on the event-driven service architecture.

## Tech

Rust + Tokio + Ratatui + portable-pty + async-openai

## TODO

This was a proof of concept. It lacks:

- [ ] .env variable loading.
- [ ] agentic loop
- [ ] tools to inject commands into the psuedoterminal (enabling the agent to not just read, but act)

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+B` | Toggle between terminal and chat mode |
| `Ctrl+Q` | Quit |
| `Ctrl+U/D` | Scroll chat up/down |
| `/clear` | Clear chat history |
| `/network` | Switch to network engineer role |
| `/linux` | Switch to Linux engineer role |

## Setup

1. Set your Azure OpenAI credentials in `src/config.rs`
2. `cargo run`
