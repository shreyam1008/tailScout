# Website hosting

The standalone TailScout website is hosted by Cloudflare Pages project `tailscout`
at https://tailscout.shreyam1008.com.np/. GitHub integration publishes `main`,
directory `docs`, with no build command and `SKIP_DEPENDENCY_INSTALL=true`.
Changes under `docs/` trigger deployments. The native apps and release assets
continue to be built and distributed through GitHub.

Keep `docs/404.html` so unknown URLs return HTTP 404 instead of a fallback home
page. Keep canonical metadata pointed at the custom domain.

`docs/_worker.js` runs in Cloudflare Pages advanced mode. It forwards ordinary
requests to the static asset binding and serves the existing `llms.txt` as
`text/markdown` when an agent requests the root with `Accept: text/markdown`;
`Vary: Accept` keeps the HTML and Markdown responses separate in caches.

GitHub Pages hosting was disabled on 9 September 2026. The former publishing
source was `main:/docs`.


Rollback now requires re-enabling and successfully deploying GitHub Pages before
restoring its DNS target; changing DNS alone is not sufficient. Retiring GitHub
Pages also retires the old github.io-hosted URLs and redirects.
