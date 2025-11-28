# Windows Service Manager

A desktop app for managing Windows services, built with Rust and Tauri.

![Screenshot](screenshots/main1.png)
![Screenshot](screenshots/main.png)

## Features

- Browse and search all Windows services
- Start, stop, pause, and resume services
- Safety: Driver services are read-only to prevent crashes
- Quick Google search for service information

## Tech Stack

- **Backend**: Rust + Windows API
- **Frontend**: Vue 3 + TypeScript + Tailwind
- **Framework**: Tauri 2

## Development
```bash
git clone https://github.com/yourusername/windows-service-manager
cd windows-service-manager
pnpm install
pnpm tauri dev
```

Requires: Rust, Node.js 18+, Windows 10/11

## What I Learned

- Windows Service Control Manager API
- FFI and unsafe Rust
- RAII pattern with Drop trait
- Tauri desktop app development

## License

MIT
