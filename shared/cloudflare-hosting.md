# Website hosting

The standalone TailScout website is hosted by Cloudflare Pages project `tailscout`
at https://tailscout.shreyam1008.com.np/. GitHub integration publishes `main`,
directory `docs`, with no build command and `SKIP_DEPENDENCY_INSTALL=true`.
Changes under `docs/` trigger deployments. The native apps and release assets
continue to be built and distributed through GitHub.

Keep `docs/404.html` so unknown URLs return HTTP 404 instead of a fallback home
page. Keep canonical metadata pointed at the custom domain.

Migration rollback: the existing GitHub Pages source is `main:/docs`; restore the
host's CNAME to `shreyam1008.github.io` with DNS-only mode if rollback is needed.
