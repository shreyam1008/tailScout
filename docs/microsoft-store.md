# Microsoft Store preparation

11 September 2026: **TailScout v0.1.4.0 passed Partner Center package validation and is saved in Submission 1; not submitted or published.** Properties, free pricing and age ratings are complete. English listing text and certification notes are saved. Screenshots and interactive Windows acceptance are pending. ProtoPeek is in certification. TailScout remains an active standalone product, even though related Tailscale workflows are available in ProtoPeek.

## Privacy-safe acceptance checkpoint

Native Windows launch and automatic refresh succeeded after the desktop became available. The app read the installed Tailscale client and displayed its current connection and device state. That screen contained real tailnet information; the owner objected to exposing it. **No capture of that screen was saved as a Store asset or uploaded to Microsoft.** Do not publish real account names, device names, addresses or tailnet identifiers.

The live-data app process was closed without changing the Tailscale connection. A separate process was launched with a temporary process-only PATH excluding the installed client. The real released app showed its missing-client guidance and no saved accounts, devices or tailnet details. System/user environment settings and the Tailscale service were not changed. This verifies the normal missing-client state; an invalid explicit binary override is a separate path and did not show the same guidance.

Capture subsequently failed with `IGraphicsCaptureItemInterop.CreateForMonitor` (0x80070057); a fresh-window retry returned `GetCursorPos failed: Access is denied` (0x80070005). No publishable PNG was captured. Keep screenshots and submission pending rather than using the private screen or a synthetic replacement. Connected actions that would alter the owner's network remain untested.

## Reserved identity

- Product ID: `9NXWT9JB492V`
- Submission 1: `1152921505701871799`
- Package name: `shreyam1008.TailScout`
- Publisher: `CN=56C87ED7-40E5-4525-B1C3-5F8CD5E02720`
- Publisher display: `shreyam1008`
- Package family: `shreyam1008.TailScout_ax0kgekbzfne6`
- [Owner dashboard](https://partner.microsoft.com/en-US/dashboard/products/9NXWT9JB492V/overview)

The reservation page requires submission within three months (11 December 2026). Do not display the reserved address as an available download until publication is verified.

## Source and release evidence

The corrected [v0.1.4 release](https://github.com/shreyam1008/tailScout/releases/tag/v0.1.4) is published. [Release CI 34610604943](https://github.com/shreyam1008/tailScout/actions/runs/34610604943) passed across Windows, macOS and Linux, including the Windows `--startup-check` which constructs MainWindow without opening a window or contacting a tailnet. Four Windows core tests also passed locally. This verifies WinUI resource loading; interactive and connected-tailnet behavior remain separate acceptance checks.

[Store workflow 34611135978](https://github.com/shreyam1008/tailScout/actions/runs/34611135978) automatically consumed that release, verified its asset digest and produced the validated `TailScout_0.1.4.0_x64.msix`. SHA-256: `720cb0014c571493a39da8726457d7e29ef9f908a0a2e9a45e21acad4c1b2992`. The downloaded package matched the receipt and passed Partner Center validation. Desktop availability is selected; future device families are disabled. The upload job was deliberately skipped.

The exact CI package was also unpacked locally and development-registered, upgrading the earlier test registration from 0.1.3.0 to 0.1.4.0 with Status Ok. Its `--startup-check` passed. It remains registered for interactive acceptance; this does not establish Store-signed install/update/uninstall behavior. The owner saved the IARC declaration and the dashboard confirms age ratings Complete. Native launch was retried after the owner's browser interaction and still returned access denied.

### Earlier diagnostic evidence (superseded by v0.1.4 above)

The current Windows app is native C#/WinUI 3 with .NET 10, `WindowsPackageType=None` and a self-contained Windows App SDK. The source explicitly labels Windows a preview. The published v0.1.3 release contains `tailscout-v0.1.3-windows-x64-winui.zip` (87,739,491 bytes); GitHub reports SHA-256 `0076914303ed97176e4c2f68632aaa90643a443b32f1de15d92269eb7b1d1229`. This inventory is not a fresh ZIP download or runtime test.

The initial NETSDK1045 blocker was resolved using official .NET SDK 10.0.401 in an isolated local tools directory, leaving the system SDK unchanged. Windows core tests passed (4 tests), and a Release win-x64 self-contained native publish succeeded into `dist/store/native-x64`. This proves compilation and core tests, not packaged WinUI launch or Store acceptance; those remain pending.

## Packaging and acceptance work

The local candidate passed MakeAppx and development registration (Status Ok, version 0.1.3.0). It uses the existing SVG logo rasterized into `packaging/windows/store/logo-512.png`; the source SVG is unchanged. The native publish initially crashed while loading MainWindow because `TailScout.Windows.pri` was missing. `EnableMsixTooling=true` restores that file in publish output. The public v0.1.3 ZIP also lacks it, so do not submit that unchanged release. The candidate includes current source plus the fix; a new stable release and post-fix native launch verification are still required.

Native computer control subsequently returned `GetCursorPos failed: Access is denied`. The owner has been asked to restore an unlocked, connected desktop. Screenshots and post-fix interactive acceptance remain pending; development registration is not evidence that the main window launches.

1. Publish a stable Windows x64 layout using .NET 10. Keep every runtime and WinUI dependency; a single copied executable is insufficient.
2. Add an MSIX manifest using the assigned identity, Windows.Desktop and the native `TailScout.Windows.exe` entry point. Validate packaged WinUI bootstrap/resource loading rather than assuming the unpackaged ZIP works unchanged.
3. Rasterize the existing `packaging/icons/hicolor/scalable/apps/dev.shre.TailScout.svg` for Store sizes without redrawing the branding. Use actual Windows screenshots, not the Linux showcase video/poster.
4. Keep `Cargo.toml` as the version source and assert the stable tag matches it. Map v0.1.4 to 0.1.4.0; leave the fourth component zero and reject rolling/prerelease tags in the stable lane.
5. Test Start launch, missing Tailscale CLI, daemon stopped, signed-out and permission-denied states. Verify Program Files/PATH discovery and `TAILSCOUT_TAILSCALE_BIN` under package identity.
6. On a real test tailnet, check status, account switching, exit nodes, connect/disconnect, diagnostics and Taildrop. Do not alter the owner's active network session merely to obtain screenshots.
7. Test package install, update and uninstall. TailScout must not imply that uninstalling it also removes the separately installed Tailscale service.
8. Finish Store privacy/support text, age ratings, free pricing, screenshots and truthful runFullTrust explanation for invoking Tailscale and accessing selected files. Clarify that TailScout is independent of Tailscale Inc. and requires the official client.

Proposed short description: **A native Windows workbench for your installed Tailscale client.**

## Automation

Follow ProtoPeek's published-stable-release package lane after Windows acceptance. CI should build MSIX and retain checksums/receipts. Store upload is a separate stage requiring Partner Center Entra API access and initial submission setup; verified Microsoft-account access alone is insufficient. Never claim automatic publishing from the presence of an artifact workflow.

Implemented `.github/workflows/store.yml`: stable release and release-workflow completion triggers, GitHub asset digest verification, required WinUI PRI/runtime files, executable version check, MSIX validation and retained receipt. Completion triggers cover releases created using GitHub's built-in token. Upload is enabled only with repository variable `STORE_UPLOAD_ENABLED=true` and environment `microsoft-store` secrets `STORE_TENANT_ID`, `STORE_SELLER_ID`, `STORE_CLIENT_ID`, `STORE_CLIENT_SECRET`. The upload preflight requires a published first submission and refuses to replace an existing draft. Credentials are not yet configured.

Partner Center currently has no associated Entra tenant. The owner declined billing/account setup for now. Keep API uploads disabled; release-to-MSIX artifact automation works independently and needs no new billing setup.

References: [MSIX requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements), [Store CLI](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/overview), [Windows source notes](../platform/windows/README.md).
