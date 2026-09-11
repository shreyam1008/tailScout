# TailScout Store copy

Short description: Inspect your Tailscale devices, accounts, exit nodes and Taildrop from a native Windows workbench.

TailScout is an independent native Windows desktop workbench for an installed Tailscale client. Inspect your devices and connection state, view account and tailnet details, choose exit nodes, use Taildrop and run diagnostics from a WinUI interface.

TailScout does not bundle or replace Tailscale. Install the official Tailscale client separately and sign in to your own account. Availability of actions depends on your Tailscale permissions, operating system and network configuration. TailScout is not affiliated with Tailscale Inc.

TailScout remains a standalone app. ProtoPeek provides related Tailscale workflows inside a broader protocol workbench; this Store app opens TailScout's own native window.

Free and open source. Windows is a preview platform. Support: https://github.com/shreyam1008/tailScout/issues

Features:
- Native Windows interface for an installed Tailscale client
- Device, connection and account inspection
- Exit-node selection and Taildrop controls
- Tailscale version and network diagnostics

Certification: no TailScout account is required. Without Tailscale installed, verify the missing-client guidance. For connected functionality, use a tester-owned Tailscale account and tailnet. runFullTrust is necessary to invoke the user's installed Tailscale CLI and access explicitly selected Taildrop files. Do not disconnect a production tailnet during testing.
