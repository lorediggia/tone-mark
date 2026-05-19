# Tone-Mark-II
An open-source patch editor and controller built in Rust, designed exclusively for Boss Katana™ MkII amplifiers.

## ─── Installation ───

### Prerequisites 

#### Arch Linux and derivatives

```bash
sudo pacman -Syu base-devel git rustup
rustup default stable
```

### Build from Source

```bash
git clone https://github.com/lorediggia/tone-mark.git
cd tone-mark

cargo build --release

```

> The compiled executable is located at `target/release/tone-mark-2`
