let renderer;
let queue = Promise.resolve();
let sequence = 0;

async function loadRenderer() {
  renderer ??= import("mermaid").then(({ default: mermaid }) => {
    mermaid.initialize({
      startOnLoad: false, securityLevel: "strict", htmlLabels: false,
      suppressErrorRendering: true, maxTextSize: 50_000, maxEdges: 500,
      theme: "default", fontFamily: "system-ui, sans-serif",
      flowchart: { htmlLabels: false },
      secure: ["secure", "securityLevel", "startOnLoad", "htmlLabels", "maxTextSize",
        "maxEdges", "suppressErrorRendering", "flowchart", "theme", "themeCSS",
        "themeVariables", "fontFamily", "layout", "look"],
    });
    return mermaid;
  });
  return renderer;
}

export function prepareDiagrams(root, language) {
  const doc = root.ownerDocument;
  for (const code of root.querySelectorAll("pre[data-line] > code.language-mermaid")) {
    const pre = code.parentElement;
    const figure = doc.createElement("figure");
    figure.className = "mermaid-block";
    figure.tabIndex = 0;
    figure.dataset.line = pre.dataset.line;
    figure.dataset.lineEnd = pre.dataset.lineEnd ?? pre.dataset.line;
    figure.dataset.quote = code.textContent.replace(/\s+/gu, " ").trim();
    figure.dataset.diagramState = "pending";
    const diagram = doc.createElement("div");
    diagram.className = "mermaid-diagram";
    diagram.setAttribute("role", "img");
    diagram.setAttribute("aria-label", language === "ja" ? "Mermaid図" : "Mermaid diagram");
    const status = doc.createElement("div");
    status.className = "mermaid-status muted";
    status.setAttribute("role", "status");
    status.textContent = language === "ja" ? "図を描画しています…" : "Rendering diagram…";
    const source = doc.createElement("details");
    source.className = "mermaid-source";
    const summary = doc.createElement("summary");
    summary.textContent = language === "ja" ? "図のソース" : "Diagram source";
    pre.replaceWith(figure);
    pre.removeAttribute("data-line"); pre.removeAttribute("data-line-end"); pre.removeAttribute("tabindex");
    source.append(summary, pre);
    figure.append(diagram, status, source);
  }
}

export function renderDiagrams(root, language, load = loadRenderer) {
  const pending = [];
  for (const figure of root.querySelectorAll('.mermaid-block[data-diagram-state="pending"]')) {
    figure.dataset.diagramState = "rendering";
    const source = figure.querySelector(".mermaid-source");
    const text = source.querySelector("code").textContent;
    const status = figure.querySelector(".mermaid-status");
    const task = queue.catch(() => {}).then(async () => {
      if (!figure.isConnected) return;
      try {
        // Enforce before loading/parsing; Mermaid otherwise replaces oversized
        // inputs with an error diagram, which would look like successful output.
        if (text.length > 50_000) throw new Error(language === "ja" ? "ソースが50,000文字を超えています。" : "Source exceeds 50,000 characters.");
        const mermaid = await load();
        if (!figure.isConnected) return;
        const { svg } = await mermaid.render(`sdev-diagram-${++sequence}`, text);
        if (!figure.isConnected) return;
        // Only Mermaid's strict renderer supplies this SVG; never bind diagram
        // callbacks or use raw Markdown as HTML.
        const template = figure.ownerDocument.createElement("template");
        template.innerHTML = svg;
        for (const link of template.content.querySelectorAll("a")) link.replaceWith(...link.childNodes);
        figure.querySelector(".mermaid-diagram").replaceChildren(template.content);
        figure.dataset.diagramState = "ready";
        status.remove();
      } catch (error) {
        if (!figure.isConnected) return;
        figure.dataset.diagramState = "error";
        source.open = true;
        status.classList.add("warning");
        status.textContent = (language === "ja" ? "図を表示できません。ソースを確認してください。 " : "Could not render diagram. Check its source. ") + String(error?.message ?? error).slice(0, 500);
      }
    });
    queue = task;
    pending.push(task);
  }
  return Promise.all(pending);
}
