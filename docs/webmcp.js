(() => {
  const context = navigator.modelContext;
  if (!context?.registerTool && !context?.provideContext) return;
  const origin = window.location.origin;
  const tools = [
    {
      name: "get_tailscout_release",
      description: "Return the current public TailScout release and its GitHub release URL.",
      inputSchema: { type: "object", properties: {}, additionalProperties: false },
      execute: () => ({ version: "0.1.4", name: "TailScout", url: "https://github.com/shreyam1008/tailScout/releases/tag/v0.1.4" })
    },
    {
      name: "open_tailscout_resource",
      description: "Open a public TailScout documentation or discovery resource.",
      inputSchema: {
        type: "object",
        properties: { resource: { type: "string", enum: ["llms", "native", "runtime", "skills", "apiCatalog", "openapi", "github"] } },
        required: ["resource"],
        additionalProperties: false
      },
      execute: (input = {}) => {
        const paths = {
          llms: "/llms.txt", native: "/native-platforms.md", runtime: "/runtime-footprint.md",
          skills: "/.well-known/agent-skills/index.json", apiCatalog: "/.well-known/api-catalog", openapi: "/openapi.json", github: "https://github.com/shreyam1008/tailScout"
        };
        const path = paths[String(input.resource || "llms")] || paths.llms;
        const url = path.startsWith("http") ? path : origin + path;
        window.open(url, "_blank", "noopener,noreferrer");
        return { url };
      }
    }
  ];
  if (context.registerTool) {
    const controller = new AbortController();
    window.addEventListener("pagehide", () => controller.abort(), { once: true });
    tools.forEach((tool) => {
      try { void context.registerTool(tool, { signal: controller.signal }); } catch {}
    });
  }
  if (context.provideContext) void context.provideContext({ tools });
})();
