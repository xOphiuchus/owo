
# 🦉 owo

**owo** is a blazingly fast, multithreaded CLI tool built in Rust that crawls your directory and packs all your source code into a single, beautifully formatted Markdown file.

Think of it like the `tree` command, but instead of just showing the names, it actually grabs the content—making it the perfect tool for feeding your entire codebase into an LLM (like ChatGPT, Claude, or Gemini) for context.

---

## ✨ Features

* **🚀 Multithreaded:** Uses `tokio` and a worker semaphore to read files in parallel.
* **🙈 Git-Aware:** Automatically respects your `.gitignore` rules.
* **🔍 Regex Filtering:** Custom ignore patterns using the `-I` flag.
* **📝 Syntax Highlighting:** Automatically detects file extensions for Markdown code blocks.
* **🦀 Built with Rust:** Memory-safe and high-performance.

---

## 🛠 Installation

### From GitHub Releases

1. Go to the [Releases](https://github.com/xOphiuchus/owo/releases) page.
2. Download the binary for your OS.
3. Move it to your path (e.g., `/usr/local/bin` on Linux/macOS).

### From Source

```bash
mkdir ~/owo
git clone https://github.com/xOphiuchus/owo.git ~/owo
cd ~/owo
cargo install --path .

```

---

## 🚀 Usage

### Basic Example

Scan the current directory and save everything to `context.md`:

```bash
owo -o context.md

```

### Advanced Filtering

Ignore specific folders and include hidden "dotfiles":

```bash
owo -I "node_modules|logs|vendor" -wdf -o backup.md

```

### Scan a Specific Path

```bash
owo -o output.md ./src/logic

```

---

## 🚩 Options & Flags

| Flag / Option | Long Name | Description | Default |
| --- | --- | --- | --- |
| `-o` | `--output` | **(Required)** The file where the Markdown is saved. | N/A |
| `-I` | `--ignore` | Pipe-separated regex of patterns to skip. | `obj|bin|build|dist|.git|.env.*` |
| `-w` | `--with-dotfiles` | Include hidden files (starts with a `.`). | `false` |
| `-h` | `--help` | Print help information. | N/A |
| `-V` | `--version` | Print version information. | N/A |

---

## 📂 Example Output

Your generated Markdown file will look like this:

## File: `src/main.rs`

```rust
use anyhow::{Context, Result};
// ... code ...
```

## File: `Cargo.toml`

```toml
[package]
name = "owo"
version = "0.1.0"
```

---

## 🤝 Contributing

Feel free to open issues or submit pull requests. If you like the project, give it a ⭐!

**Author:** xOphiuchus

- Made with 💌 for Open source community

**License:** APACHE-2.0

---