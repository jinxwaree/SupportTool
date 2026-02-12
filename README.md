# Emporium Support Tool

A Windows system diagnostics and support utility built with Rust and [egui](https://github.com/emilk/egui). Designed for quick hardware/software auditing and common support tasks.

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white)

## Features

- **System Info** — OS, CPU, GPU, RAM, motherboard, storage usage, and network adapters
- **HWID** — CPU ID, motherboard serial, BIOS serial, machine GUID, product ID, UUID, and disk serials
- **Windows Security** — View and toggle UAC, Real-Time Protection, SmartScreen, Defender status, and Secure Boot
- **Booster Detection** — Detects Razer Cortex, Dragon Center, and MSI Afterburner
- **Anti-Cheat Detection** — Detects Vanguard, FaceIT, ESEA, EasyAntiCheat, BattlEye, and Malwarebytes
- **Installs** — Check and install missing VC++ Redistributables and DirectX Runtime
- **Controls** — Hyper-V management, driver fixes, network reset, Windows Update blocking, and quick links to common downloads
- **Export** — Copy all system info to clipboard in one click

## Building

Requires [Rust](https://rustup.rs/) toolchain.

```bash
cargo build --release
```

The binary will be at `target/release/support_tool.exe`.

## License

All rights reserved.
