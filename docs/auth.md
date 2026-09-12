# auth.md

TailScout's public site contains documentation for a local native client. Its
discovery resources are anonymous and require no account, API key, OAuth token,
or agent registration.

The application reads the user's local Tailscale CLI and LocalAPI. The public
site does not expose a hosted tailnet API, remote MCP transport, or OAuth
authorization server, and it never publishes peer names, IP addresses,
accounts, or diagnostics. Keep those values inside the local client.

```yaml
agent_auth:
  registration_required: false
  identity_types_supported: [anonymous]
  credential_types_supported: [none]
  protected_resources: []
  authorization_servers: []
```
