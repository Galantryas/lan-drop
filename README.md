# LAN Drop & Peer Finder

A desktop app for moving things between machines on the same network, without a USB stick, a cloud account, or an internet connection.

It discovers peers on your subnet, shows them as cards, and lets you send files or share a clipboard entry with one of them. Discovery, transfer, and receipt all happen locally over the LAN.

## Features

- **Peer discovery** — scans the local subnet and lists reachable devices with hostname, IP, and response time.
- **File transfer** — send one or more files to a selected peer over an embedded HTTP server; the receiving side sees progress in a drawer.
- **Clipboard sharing** — push a text snippet to another machine, with a modal for previewing what arrives.
- **Network cut** — temporarily drops a chosen peer's connectivity, useful when you administer the LAN and need to take a device off it. Every cut is tracked and can be reverted.
- **Live feedback** — toast notifications for discovery, transfer, and errors.

## Stack

| Layer | Choice |
| --- | --- |
| Shell | Tauri 2 |
| Frontend | Svelte 5, TypeScript, Vite, Tailwind, lucide icons |
| Backend | Rust — subnet scanner, UDP discovery, HTTP transfer server |
| Platform | Windows and Linux desktop |

## Running

Prerequisites: Node 20+, Rust toolchain, and the Tauri prerequisites for your platform.

```bash
npm install
npm run tauri dev
```

Build:

```bash
npm run tauri build
```

Both machines need the app running for peer discovery to see them, and the OS firewall must allow the discovery and transfer ports on private networks.

## Project layout

```
src/                     Svelte frontend
  App.svelte             Layout and state
  PeerGrid.svelte        Discovered peers
  TransferDrawer.svelte  In-flight transfers
  ClipboardModal.svelte  Clipboard sharing
src-tauri/src/
  discovery.rs           UDP broadcast and peer registry
  scanner.rs             Subnet scan
  server.rs              File transfer HTTP server
  netcut.rs              Network cut and restore
  commands.rs            Tauri command surface
```

## Notes

Network cut affects other people's devices on the network. Use it only on networks you administer, and only on devices you are responsible for. Some operations need elevated privileges.

## Licence

Personal project. Not licensed for redistribution.
