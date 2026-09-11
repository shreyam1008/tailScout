# Microsoft Store preparation

11 September 2026: **TailScout reserved as a separate MSIX app; not submitted or published.** ProtoPeek is the first Store submission in progress. TailScout remains an active standalone product, even though related Tailscale workflows are available in ProtoPeek.

## Reserved identity

- Product ID: `9NXWT9JB492V`
- Package name: `shreyam1008.TailScout`
- Publisher: `CN=56C87ED7-40E5-4525-B1C3-5F8CD5E02720`
- Publisher display: `shreyam1008`
- Package family: `shreyam1008.TailScout_ax0kgekbzfne6`
- [Owner dashboard](https://partner.microsoft.com/en-US/dashboard/products/9NXWT9JB492V/overview)

The reservation page requires submission within three months (11 December 2026). Do not display the reserved address as an available download until publication is verified.

## Source and release evidence

The current Windows app is native C#/WinUI 3 with .NET 10, `WindowsPackageType=None` and a self-contained Windows App SDK. The source explicitly labels Windows a preview. The published v0.1.3 release contains `tailscout-v0.1.3-windows-x64-winui.zip` (87,739,491 bytes); GitHub reports SHA-256 `0076914303ed97176e4c2f68632aaa90643a443b32f1de15d92269eb7b1d1229`. This inventory is not a fresh ZIP download or runtime test.

Attempted Windows core tests on the preparation host failed before compilation with NETSDK1045: installed SDK 9.0.317 cannot target net10.0. Install/use a .NET 10 SDK or run the existing Windows CI before claiming a passing test. Do not lower the target framework to make packaging appear successful.

## Packaging and acceptance work

1. Publish a stable Windows x64 layout using .NET 10. Keep every runtime and WinUI dependency; a single copied executable is insufficient.
2. Add an MSIX manifest using the assigned identity, Windows.Desktop and the native `TailScout.Windows.exe` entry point. Validate packaged WinUI bootstrap/resource loading rather than assuming the unpackaged ZIP works unchanged.
3. Rasterize the existing `packaging/icons/hicolor/scalable/apps/dev.shre.TailScout.svg` for Store sizes without redrawing the branding. Use actual Windows screenshots, not the Linux showcase video/poster.
4. Keep `Cargo.toml` as the version source and assert the stable tag matches it. Map v0.1.3 to 0.1.3.0; leave the fourth component zero and reject rolling/prerelease tags in the stable lane.
5. Test Start launch, missing Tailscale CLI, daemon stopped, signed-out and permission-denied states. Verify Program Files/PATH discovery and `TAILSCOUT_TAILSCALE_BIN` under package identity.
6. On a real test tailnet, check status, account switching, exit nodes, connect/disconnect, diagnostics and Taildrop. Do not alter the owner's active network session merely to obtain screenshots.
7. Test package install, update and uninstall. TailScout must not imply that uninstalling it also removes the separately installed Tailscale service.
8. Finish Store privacy/support text, age ratings, free pricing, screenshots and truthful runFullTrust explanation for invoking Tailscale and accessing selected files. Clarify that TailScout is independent of Tailscale Inc. and requires the official client.

Proposed short description: **A native Windows workbench for your installed Tailscale client.**

## Automation

Follow ProtoPeek's published-stable-release package lane after Windows acceptance. CI should build MSIX and retain checksums/receipts. Store upload is a separate stage requiring Partner Center Entra API access and initial submission setup; verified Microsoft-account access alone is insufficient. Never claim automatic publishing from the presence of an artifact workflow.

References: [MSIX requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements), [Store CLI](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/overview), [Windows source notes](../platform/windows/README.md).
