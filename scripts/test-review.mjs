import assert from "node:assert/strict";
import { test } from "node:test";
import { readFile } from "node:fs/promises";
import { Window } from "happy-dom";
import { prepareDiagrams, renderDiagrams } from "../src/review/mermaid.js";

const bundle = await readFile(new URL("../src/review/viewer.bundle.js", import.meta.url), "utf8");
const snapshot = {
  version: 1, id: "test-snapshot", captured_at: 1, base: "a".repeat(40), target: "working-tree", language: "ja", title: "Test review",
  files: [
    { path: "docs/requirements.md", role: "requirements", changed: true, old: "# Before\n", new: "# Intent\n\nKeep intent.\n", old_html: '<h1 data-line="1">Before</h1>', new_html: '<h1 data-line="1">Intent</h1><p data-line="3" tabindex="0">Keep intent.</p>' },
    { path: "docs/design.md", role: "design", changed: false, old: "Method\n", new: "Method\n", old_html: '<p data-line="1">Method</p>', new_html: '<p data-line="1">Method</p>' },
    { path: "src/code.txt", role: "implementation", changed: true, old: "before\n", new: "after\n" },
    { path: "src/deleted.txt", role: "implementation", changed: true, old: "removed\n", new: null },
  ],
  warnings: [],
  items: [{ id: "ITEM1", title: "Is the intent preserved?", attention: true, question: "Question", refs: [{ id: "R1", label: "Keep the user's intent", path: "docs/requirements.md", line: 3, side: "new" }, { id: "D1", label: "Choose this method", path: "docs/design.md", line: 1, side: "new" }, { id: "C1", label: "Implement the method", path: "src/code.txt", line: 1, side: "new" }], links: [{ from: "R1", to: "D1", reason: "This method preserves the required intent." }, { from: "D1", to: "C1", reason: "This branch implements the selected method." }], decisions: [{ text: "A decision", basis: "unknown", reason: "Unknown" }], examples: [{ label: "Example", kind: "illustrative", input: "input", result: "result" }] }],
};

async function page(data = snapshot, fetcher = null, setup = null) {
  // Execute only the locally built viewer and these fixed test fixtures.
  const window = new Window({ url: "https://review.test", settings: {
    enableJavaScriptEvaluation: true, suppressInsecureJavaScriptEnvironmentWarning: true,
  } });
  const errors = [];
  // Happy DOM lacks this standard browser API; retain the real clone semantics.
  window.structuredClone = structuredClone;
  window.addEventListener("error", (event) => errors.push(event.error));
  window.console.error = (...args) => errors.push(args.join(" "));
  window.fetch = fetcher ?? (() => { throw new Error("Viewer must not fetch resources"); });
  window.document.body.innerHTML = '<div id="app"></div><script id="review-data" type="application/json"></script>';
  window.document.getElementById("review-data").textContent = JSON.stringify(data);
  setup?.(window);
  window.eval(bundle);
  await window.happyDOM.waitUntilComplete();
  return { window, document: window.document, errors, close: () => window.happyDOM.abort() };
}

function click(document, selector, text) {
  const node = [...document.querySelectorAll(selector)].find((n) => n.textContent.includes(text));
  assert.ok(node, `Missing ${selector}: ${text}`); node.click(); return node;
}


function paragraph(p, line = 3) { return p.document.querySelector(`.pane.current .markdown [data-line="${line}"]`); }
function key(p, node, value, options = {}) {
  const event = new p.window.KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true, ...options });
  node.dispatchEvent(event); return event;
}
function inputComment(p, text) {
  const input = p.document.activeElement?.tagName === "TEXTAREA" ? p.document.activeElement : p.document.querySelector("textarea");
  assert.ok(input); input.value = text; input.dispatchEvent(new p.window.Event("input")); return input;
}
async function exportedNotes(p, format = "JSON") {
  let exported;
  p.window.URL.createObjectURL = (blob) => { exported = blob; return "blob:test"; };
  p.window.URL.revokeObjectURL = () => {};
  p.window.HTMLAnchorElement.prototype.click = function () {};
  click(p.document, "button", format === "JSON" ? "JSONを書き出す" : "メモを書き出す");
  const value = await exported.text(); return format === "JSON" ? JSON.parse(value) : value;
}
function hover(p, node) { node.dispatchEvent(new p.window.MouseEvent("mouseenter")); }

test("Mermaid figures retain source targets, render sequentially, and ignore detached results", async () => {
  const window = new Window();
  const root = window.document.createElement("div"); window.document.body.append(root);
  root.innerHTML = '<pre data-line="4" data-line-end="7" tabindex="0"><code class="language-mermaid">flowchart LR\n A--&gt;B\n</code></pre><pre data-line="9"><code>const value = 1;</code></pre>';
  prepareDiagrams(root, "ja"); prepareDiagrams(root, "ja");
  const figure = root.querySelector("figure");
  assert.equal(root.querySelectorAll("figure").length, 1);
  assert.equal(figure.dataset.quote, "flowchart LR A-->B");
  assert.equal(figure.dataset.line, "4"); assert.equal(figure.dataset.lineEnd, "7");
  assert.equal(root.querySelector('pre[data-line="9"]').textContent, "const value = 1;");
  assert.equal(figure.querySelector("pre").hasAttribute("data-line"), false);
  let finish, started;
  const entered = new Promise((resolve) => { started = resolve; });
  const drawn = renderDiagrams(root, "ja", async () => ({ render: (id, source) => {
    assert.match(id, /^sdev-diagram-/); assert.equal(source, "flowchart LR\n A-->B\n"); started();
    return new Promise((resolve) => { finish = resolve; });
  } }));
  await entered;
  figure.remove(); finish({ svg: '<svg><text>Old result</text></svg>' }); await drawn;
  assert.equal(figure.querySelector("svg"), null);
  root.append(figure); figure.dataset.diagramState = "pending";
  await renderDiagrams(root, "ja", async () => ({ render: async () => ({ svg: '<svg><a href="https://example.com"><text>New result</text></a></svg>' }) }));
  assert.equal(figure.dataset.diagramState, "ready");
  assert.equal(figure.querySelector("svg text").textContent, "New result");
  assert.equal(figure.querySelector("a"), null);
  assert.equal(figure.querySelector(".mermaid-source").open, false);
  assert.equal(figure.dataset.quote, "flowchart LR A-->B");
  await window.happyDOM.abort();
});

test("failed or oversized Mermaid keeps literal source and does not stop the next diagram", async () => {
  const window = new Window(); const root = window.document.createElement("div"); window.document.body.append(root);
  for (const [index, source] of ['bad <script>alert(1)</script>', 'a'.repeat(50_001), 'flowchart LR; A-->B'].entries()) {
    const pre = window.document.createElement("pre"); pre.dataset.line = String(index + 1);
    const code = window.document.createElement("code"); code.className = "language-mermaid"; code.textContent = source; pre.append(code); root.append(pre);
  }
  prepareDiagrams(root, "en");
  let calls = 0;
  await renderDiagrams(root, "en", async () => ({ render: async (_id, source) => {
    calls++; if (source.startsWith("bad")) throw new Error("Invalid syntax <img src=x>");
    return { svg: "<svg></svg>" };
  } }));
  const figures = [...root.querySelectorAll("figure")];
  assert.equal(calls, 2); assert.deepEqual(figures.map((node) => node.dataset.diagramState), ["error", "error", "ready"]);
  assert.equal(figures[0].querySelector(".mermaid-source").open, true);
  assert.equal(figures[0].querySelector("code").textContent, 'bad <script>alert(1)</script>');
  assert.equal(root.querySelector("script, img"), null);
  assert.match(figures[1].querySelector(".mermaid-status").textContent, /50,000/);
  await window.happyDOM.abort();
});

test("the initial surface is the specification; selecting text reveals its examples and design reasons", async () => {
  const input = structuredClone(snapshot);
  input.items[0].assessment = { kind: "missing_design", summary: "A choice to record", impact: "A user consequence", inspected: "Changed behavior and specifications" };
  const p = await page(input);
  try {
    assert.ok(paragraph(p).textContent.includes("Keep intent."));
    assert.equal(p.document.querySelector(".relationship-map"), null);
    assert.equal(p.document.querySelector(".inspector"), null);
    assert.equal(p.document.querySelector(".explanation-popover"), null);
    assert.ok(!p.document.getElementById("app").textContent.includes("src/code.txt"));
    paragraph(p).click();
    const details = p.document.querySelector(".explanation-popover");
    assert.ok(details.textContent.includes("This method preserves the required intent."));
    assert.ok(details.textContent.includes("A user consequence"));
    assert.ok(details.textContent.includes("説明用の例"));
    assert.ok(details.textContent.includes("理由不明"));
    assert.ok(details.querySelector(".investigation-details"));
    click(p.document, ".passage-connection button", "Choose this method");
    await p.window.happyDOM.waitUntilComplete();
    assert.equal(p.document.querySelector(".pane.current .document-card").dataset.path, "docs/design.md");
    p.document.querySelector('[aria-label="戻る"]').click();
    assert.equal(p.document.querySelector(".pane.current .document-card").dataset.path, "docs/requirements.md");
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

test("click focuses details, and typing starts an inline comment without losing the initial character", async () => {
  const p = await page();
  try {
    const text = paragraph(p); text.click();
    assert.equal(p.document.activeElement, text);
    assert.ok(p.document.querySelector(".explanation-popover"));
    assert.equal(p.document.querySelector("textarea"), null, "A click must not start commenting");
    key(p, text, "k", { metaKey: true }); assert.equal(p.document.querySelector("textarea"), null);
    assert.equal(key(p, text, "P").defaultPrevented, true);
    assert.equal(p.document.querySelector("textarea").value, "P");
    assert.equal(p.document.activeElement, text.nextElementSibling.querySelector("textarea"));
    assert.equal(p.document.querySelector(".explanation-popover"), null);
    inputComment(p, "Please clarify this requirement.");
    click(p.document, ".stack-tab", "Design");
    assert.equal(p.document.querySelector("textarea"), null);
    p.document.querySelector('[aria-label="戻る"]').click();
    assert.equal(p.document.querySelector("textarea").value, "Please clarify this requirement.");
    click(p.document, ".inline-thread button", "コメントを追加");
    assert.ok(paragraph(p).nextElementSibling.textContent.includes("Please clarify"));
    const report = await exportedNotes(p);
    assert.deepEqual(report.notes[0].reference, { path: "docs/requirements.md", line: 3, side: "new" });
    assert.equal(report.notes[0].item, "ITEM1");
    click(p.document, ".note button", "削除"); assert.equal(p.document.querySelector(".note"), null);
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

test("a focused passage starts commenting with Enter, while Enter in its editor stays multiline", async () => {
  const p = await page();
  try {
    const text = paragraph(p); text.focus();
    assert.equal(p.document.querySelector(".explanation-popover"), null);
    key(p, text, "Enter");
    assert.equal(p.document.querySelector("textarea").value, "");
    const input = inputComment(p, "One line\nAnother line");
    assert.equal(key(p, input, "Enter").defaultPrevented, false);
    click(p.document, ".inline-thread button", "コメントを追加");
    assert.ok(text.nextElementSibling.textContent.includes("Another line"));
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

test("hover and passive focus never open or replace details; explicit selection stays until dismissed", async () => {
  const p = await page();
  try {
    const text = paragraph(p);
    const heading = p.document.querySelector('.markdown h1[data-line="1"]');
    for (const node of [text, text.querySelector(".review-marker")]) {
      hover(p, node); node.focus();
      assert.equal(p.document.querySelector(".explanation-popover"), null);
    }
    text.click();
    const details = p.document.querySelector(".explanation-popover");
    hover(p, heading); heading.focus();
    text.dispatchEvent(new p.window.MouseEvent("mouseleave"));
    details.dispatchEvent(new p.window.MouseEvent("mouseleave"));
    await new Promise((resolve) => setTimeout(resolve, 210));
    assert.equal(p.document.querySelector(".explanation-popover"), details);
    key(p, text, "Escape");
    assert.equal(p.document.querySelector(".explanation-popover"), null);
    assert.equal(p.document.activeElement, text);
    hover(p, text); assert.equal(p.document.querySelector(".explanation-popover"), null);
    text.querySelector(".review-marker").click();
    assert.ok(p.document.querySelector(".explanation-popover"));
    p.document.querySelector(".comparison-summary").click();
    assert.equal(p.document.querySelector(".explanation-popover"), null);
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

test("headings expose material in their section without inferring new relationships", async () => {
  const p = await page();
  try {
    const heading = p.document.querySelector('.markdown h1[data-line="1"]');
    assert.ok(heading.querySelector(".review-marker"));
    heading.click();
    assert.ok(p.document.querySelector(".popover-heading").textContent.includes("この節の判断材料"));
    assert.equal(p.document.querySelectorAll(".passage-connection").length, 1);
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

test("Pierre line clicks focus details, then typing comments on the exact old/new side", async () => {
  const p = await page();
  try {
    click(p.document, ".segmented", "差分"); await p.window.happyDOM.waitUntilComplete();
    const container = p.document.querySelector("diffs-container");
    assert.ok(container.shadowRoot.textContent.includes("Before"));
    const line = container.shadowRoot.querySelector('[data-line="1"][data-line-type="change-deletion"]');
    line.click(); await p.window.happyDOM.waitUntilComplete();
    assert.equal(p.document.querySelector("textarea"), null);
    const focused = p.document.activeElement;
    assert.ok(focused.classList.contains("diff-line-focus"));
    assert.ok(p.document.querySelector(".explanation-popover"));
    key(p, focused, "W");
    assert.equal(p.document.querySelector("textarea").value, "W");
    inputComment(p, "Why was this removed?");
    click(p.document, ".inline-thread button", "コメントを追加");
    await p.window.happyDOM.waitUntilComplete();
    const report = await exportedNotes(p);
    assert.equal(report.notes[0].reference.side, "old"); assert.equal(report.notes[0].reference.line, 1);
    assert.ok(container.querySelector(".note").textContent.includes("Why was this removed?"));
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

test("questions without a specification accept item comments and Markdown export", async () => {
  const input = structuredClone(snapshot); input.items[0].refs = []; input.items[0].links = [];
  const p = await page(input);
  try {
    assert.ok(p.document.querySelector(".unanchored-questions .question").textContent.includes("Question"));
    click(p.document, ".unanchored-questions button", "コメントを書く");
    inputComment(p, "Record this rule."); click(p.document, ".inline-thread button", "コメントを追加");
    const report = await exportedNotes(p);
    assert.equal(report.notes[0].reference, null); assert.equal(report.notes[0].item, "ITEM1");
    assert.ok((await exportedNotes(p, "Markdown")).includes("Record this rule."));
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

test("empty and unlinked states do not imply a semantic review or invent connection reasons", async () => {
  const unlinked = structuredClone(snapshot); unlinked.items = [];
  const p = await page(unlinked);
  try {
    assert.ok(p.document.querySelector(".sidebar").textContent.includes("Agentの分析はまだありません"));
    assert.equal(p.document.querySelectorAll(".review-marker").length, 0);
    paragraph(p).click(); assert.ok(p.document.querySelector(".explanation-popover").textContent.includes("まだありません"));
  } finally { await p.close(); }
  const empty = structuredClone(unlinked); empty.files = [];
  const e = await page(empty);
  try { assert.ok(e.document.querySelector(".empty-state").textContent.includes("変更はありません")); } finally { await e.close(); }
  const legacy = structuredClone(snapshot); delete legacy.items[0].links;
  const old = await page(legacy);
  try {
    paragraph(old).click();
    assert.ok(old.document.querySelector(".trace-missing-links").textContent.includes("それだけで仕様の問題とは判断できません"));
    assert.equal(old.document.querySelectorAll(".passage-connection").length, 0);
  } finally { await old.close(); }
});

test("a reference in the middle of a paragraph reveals its own explanation at that paragraph", async () => {
  const input = structuredClone(snapshot);
  input.files[0].new = "# Intent\n\nFirst sentence.\nMiddle sentence.\nLast sentence.\n";
  input.files[0].new_html = '<h1 data-line="1" data-line-end="1">Intent</h1><p data-line="3" data-line-end="5">First sentence. Middle sentence. Last sentence.</p>';
  input.items[0].refs[0].line = 4;
  const p = await page(input);
  try {
    paragraph(p).click();
    assert.ok(p.document.querySelector(".explanation-popover").textContent.includes("Example"));
    click(p.document, ".explanation-popover button", "コメントを書く");
    inputComment(p, "Clarify the middle sentence."); click(p.document, ".inline-thread button", "コメントを追加");
    assert.ok(paragraph(p).nextElementSibling.querySelector(".note").textContent.includes("middle sentence"));
    const report = await exportedNotes(p); assert.equal(report.notes[0].reference.line, 4);
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

test("back and forward retain document reading positions", async () => {
  const p = await page();
  try {
    p.window.scrollTo({ top: 360 }); click(p.document, ".stack-tab", "Design");
    await p.window.happyDOM.waitUntilComplete(); p.window.scrollTo({ top: 80 });
    p.document.querySelector('[aria-label="戻る"]').click(); await p.window.happyDOM.waitUntilComplete();
    assert.equal(p.window.scrollY, 360);
    p.document.querySelector('[aria-label="進む"]').click(); await p.window.happyDOM.waitUntilComplete();
    assert.equal(p.window.scrollY, 80); assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});


test("live related passages use the local endpoint and cache repeated reading without generated rationale", async () => {
  const input = structuredClone(snapshot); input.live = true; input.items = [];
  const calls = [];
  const evidence = "😀 An actual design passage";
  const passage = `${evidence} <script>ignored</script>`;
  const p = await page(input, async (url, options) => {
    calls.push({url,body:JSON.parse(options.body)});
    return { ok:true,json:async () => ({ matches:[{path:"docs/design.md",line:1,side:"new",text:passage,heading:"Chosen method",probability:.91,highlights:[{start:0,end:evidence.length,text:evidence},{start:-1,end:100,text:"invented"}]}],tested:1,total:1,partial:false,model:"jev-test",cache:{hits:0,misses:1} }) };
  });
  try {
    assert.equal(p.document.querySelectorAll(".review-marker, .review-linked").length, 0, "Live reading needs no per-paragraph related-action decoration");
    hover(p, paragraph(p)); paragraph(p).focus();
    await p.window.happyDOM.waitUntilComplete();
    assert.equal(calls.length, 0, "Pointer movement and passive focus must not request judgments");
    paragraph(p).click(); await p.window.happyDOM.waitUntilComplete();
    const popup = p.document.querySelector(".explanation-popover");
    assert.ok(popup.classList.contains("inline-peek"));
    assert.ok(popup.closest(".markdown"));
    assert.ok(popup.querySelector(".related-passage").textContent.includes("An actual design passage"));
    assert.equal(popup.querySelector(".related-passage").textContent, passage, "Keep surrounding text intact");
    assert.equal(popup.querySelectorAll("mark").length, 1);
    assert.equal(popup.querySelector("mark").textContent, evidence);
    assert.equal(popup.querySelector(".live-related h3"), null);
    assert.equal(popup.querySelector(".popover-heading strong"), null);
    assert.equal(popup.querySelector("script"), null);
    assert.equal(calls.length,1); assert.equal(calls[0].url,"/api/related");
    assert.deepEqual(calls[0].body,{path:"docs/requirements.md",line:3,side:"new"});
    key(p,paragraph(p),"Escape"); paragraph(p).click(); await p.window.happyDOM.waitUntilComplete();
    assert.equal(calls.length,1,"Returning to the same paragraph reuses its result");
    const mode = p.document.querySelector(".reading-mode"); mode.value="popover"; mode.dispatchEvent(new p.window.Event("change"));
    assert.equal(p.document.querySelector(".explanation-popover").parentElement,p.document.body);
    assert.deepEqual(p.errors,[]);
  } finally { await p.close(); }
});

test("table reading keeps rows, headers, comments and related row emphasis intact", async () => {
  const input=structuredClone(snapshot);input.live=true;input.items=[];
  input.files[0].new="# Table\n\n| 操作 | 結果 |\n| --- | --- |\n| コピー | メモをコピーできる。 |\n| 読む | 文ごとに選べる。 |\n";
  input.files[0].new_html='<h1 data-line="1">Table</h1><table><thead><tr tabindex="0" data-line="3" data-line-end="3"><th>操作</th><th>結果</th></tr></thead><tbody><tr tabindex="0" data-line="5" data-line-end="5"><td><code>コピー</code></td><td>メモをコピーできる。</td></tr><tr tabindex="0" data-line="6" data-line-end="6"><td>読む</td><td>文ごとに選べる。</td></tr></tbody></table>';
  const requests=[];
  const p=await page(input,async(url,options)=>{requests.push(JSON.parse(options.body));return {ok:true,json:async()=>({matches:[{path:"docs/design.md",line:1,side:"new",heading:"選択方法",text:"table source",probability:.9,table:{headers:["操作","方法"],rows:[["コピー","Clipboard API"],["読む","<img src=x onerror=alert(1)>"]],highlighted_rows:[1]}}],tested:1,total:1,model:"jev-test",cache:{hits:0,misses:1}})};});
  try {
    const first=paragraph(p,5);first.click();await p.window.happyDOM.waitUntilComplete();
    assert.deepEqual(requests[0],{path:"docs/requirements.md",line:5,side:"new",quote:"コピー | メモをコピーできる。"});
    const source=p.document.querySelector(".pane.current .markdown>table");
    assert.equal(source.querySelector(".comment-anchor,.explanation-popover"),null,"Details stay outside table markup");
    assert.ok(p.document.querySelector(".explanation-popover").querySelector(".related-table"));
    const related=p.document.querySelector(".related-table");
    assert.deepEqual([...related.querySelectorAll("th")].map(n=>n.textContent),["操作","方法"]);
    assert.equal(related.querySelectorAll("tbody tr").length,2);
    assert.equal(related.querySelectorAll(".related-row-highlight").length,1);
    assert.equal(related.querySelector(".related-row-highlight").firstChild.textContent,"読む");
    assert.equal(related.querySelector("img"),null);
    assert.ok(related.textContent.includes("<img src=x onerror=alert(1)>"));
    key(p,first,"j");await p.window.happyDOM.waitUntilComplete();
    assert.equal(p.document.activeElement.dataset.line,"6");
    key(p,p.document.activeElement,"Enter");
    const input=p.document.querySelector("textarea");assert.ok(input);assert.equal(input.closest("table"),null);
    assert.equal(p.document.querySelector(".comment-quote").textContent,"読む | 文ごとに選べる。");
    assert.ok(source.querySelector("code"),"Inline formatting remains in the source");
    assert.deepEqual(p.errors,[]);
  } finally {await p.close();}
});

test("failed sentence emphasis keeps the paragraph readable and retries without inventing highlights", async () => {
  const input=structuredClone(snapshot);input.live=true;input.items=[];let calls=0;
  const text="Keep the context. This is the method.";
  const p=await page(input,async()=>({ok:true,json:async()=>({matches:[{path:"docs/design.md",line:1,side:"new",text,heading:"Method",probability:.9,highlights:[]}],tested:1,total:1,model:"jev-test",cache:{hits:1,misses:0},highlight_error:++calls===1 ? "Unavailable" : null})}));
  try {
    paragraph(p).click();await p.window.happyDOM.waitUntilComplete();
    assert.equal(p.document.querySelector(".related-passage").textContent,text);
    assert.equal(p.document.querySelector(".related-passage mark"),null);
    click(p.document,".live-related button","強調を再取得");await p.window.happyDOM.waitUntilComplete();
    assert.equal(calls,2);
    assert.equal(p.document.querySelector(".related-passage").textContent,text);
    assert.ok(!p.document.querySelector(".live-related").textContent.includes("対応箇所の強調を取得できませんでした"));
    assert.deepEqual(p.errors,[]);
  } finally { await p.close(); }
});

test("J/K move between source blocks and never start comments; input controls retain their own keys", async () => {
  const p=await page();
  try {
    key(p,p.document.body,"j"); assert.equal(p.document.activeElement.dataset.line,"1");
    key(p,p.document.activeElement,"j"); assert.equal(p.document.activeElement.dataset.line,"3");
    assert.equal(p.document.querySelector("textarea"),null);
    key(p,p.document.activeElement,"k"); assert.equal(p.document.activeElement.dataset.line,"1");
    key(p,p.document.activeElement,"Enter");
    const input=p.document.querySelector("textarea"); assert.ok(input);
    assert.equal(key(p,input,"j").defaultPrevented,false);
    assert.equal(p.document.activeElement,input);
    assert.deepEqual(p.errors,[]);
  } finally { await p.close(); }
});

test("an API failure remains retryable and distinct from no related passage", async () => {
  const input=structuredClone(snapshot);input.live=true;input.items=[];let calls=0;
  const p=await page(input,async () => { calls++; return calls===1 ? {ok:false,json:async()=>({error:"Jev unavailable"})} : {ok:true,json:async()=>({matches:[],tested:2,total:2,partial:false,model:"jev-test",cache:{hits:0,misses:1}})}; });
  try {
    paragraph(p).click();await p.window.happyDOM.waitUntilComplete();
    assert.ok(p.document.querySelector(".live-related .warning").textContent.includes("Jev unavailable"));
    click(p.document,".live-related button","再試行");await p.window.happyDOM.waitUntilComplete();
    assert.equal(calls,2);assert.ok(p.document.querySelector(".live-related").textContent.includes("見つかりませんでした"));
    assert.deepEqual(p.errors,[]);
  } finally { await p.close(); }
});

test("code checking shows its actual inputs, two judgments, progress, and candidate diffs", async () => {
  const input=structuredClone(snapshot);input.live=true;input.items=[];
  input.code_scope={files:[{path:"src/code.txt",regions:1,notice:null}],total:1,specifications:[{path:"docs/requirements.md",role:"requirements"},{path:"docs/design.md",role:"design"}],partial:false};
  let calls=0;
  const p=await page(input,async (url) => { calls++; assert.equal(url,"/api/uncovered");return {ok:true,json:async()=>({candidates:[{path:"src/code.txt",line:1,side:"new",diff:"-before\n+after\n",kind:"uncertain",needs_spec:.9,covered:.45}],outcomes:[{path:"src/code.txt",line:1,kind:"uncertain",needs_spec:.9,covered:.45}],scanned:1,total:1,next_cursor:null,skipped_files:1,partial:true})}; });
  try {
    click(p.document,".actions button","仕様にない実装");
    assert.equal(calls,0,"Opening the input summary does not start a model request");
    assert.ok(p.document.querySelector(".scan-inputs").textContent.includes("src/code.txt"));
    assert.ok(p.document.querySelector(".scan-inputs").textContent.includes("docs/requirements.md"));
    assert.ok(p.document.querySelector(".scan-questions").textContent.includes("今のRequirements／Design"));
    assert.ok(p.document.querySelector(".scan-progress").textContent.includes("未検査"));
    click(p.document,".scan-panel button","変更したコードを調べる");await p.window.happyDOM.waitUntilComplete();
    assert.ok(p.document.querySelector(".candidate-diff").textContent.includes("+after"));
    assert.ok(p.document.querySelector(".candidate-judgments").textContent.includes("はい"));
    assert.ok(p.document.querySelector(".candidate-judgments").textContent.includes("判断保留"));
    assert.ok(p.document.querySelector(".scan-progress").textContent.includes("照合完了 · 1/1"));
    assert.ok(p.document.querySelector(".scan-recent").textContent.includes("src/code.txt:1"));
    assert.ok(p.document.querySelector(".scan-panel .warning"));
    assert.equal(p.document.querySelector(".trace-implementation"),null);
    click(p.document,".scan-spec-link","Requirements");
    assert.ok(p.document.querySelector(".pane.current .document-card").dataset.path.includes("requirements"));
    assert.deepEqual(p.errors,[]);
  } finally { await p.close(); }
});

test("stopping a scan waits for its active comparison and retains a resumable incomplete result", async()=>{
  const input=structuredClone(snapshot);input.live=true;input.items=[];
  input.code_scope={files:[{path:"src/code.txt",regions:8}],total:8,specifications:[],partial:false};
  let respond; let calls=0;
  const p=await page(input,async()=>{ calls++; return new Promise((resolve)=>{respond=resolve;}); });
  try {
    click(p.document,".actions button","仕様にない実装");
    click(p.document,".scan-panel button","変更したコードを調べる");
    await Promise.resolve(); await Promise.resolve();
    click(p.document,".scan-panel button","この照合で停止");
    assert.ok(p.document.querySelector(".scan-panel .button.primary").disabled);
    respond({ok:true,json:async()=>({candidates:[],outcomes:[{kind:"covered"}],scanned:4,total:8,next_cursor:4,skipped_files:0,partial:false})});
    await p.window.happyDOM.waitUntilComplete();
    assert.equal(calls,1);
    assert.ok(p.document.querySelector(".scan-progress").textContent.includes("停止・未完了 · 4/8"));
    assert.ok(p.document.querySelector(".scan-panel .button.primary").textContent.includes("続きから調べる"));
    assert.deepEqual(p.errors,[]);
  } finally { await p.close(); }
});



test("sentences retain formatting, have separate relation queries and comment targets on the same source line", async () => {
  const input = structuredClone(snapshot); input.live = true; input.items = [];
  input.files[0].new = "# Intent\n\nクリックで**詳細**を開きます。ホバーでは変えません。\n";
  input.files[0].new_html = '<h1 data-line="1">Intent</h1><p data-line="3" data-line-end="3">クリックで<strong>詳細</strong>を開きます。ホバーでは変えません。</p>';
  const calls = [];
  const p = await page(input, async (url, options) => {
    calls.push(JSON.parse(options.body));
    return {ok:true,json:async()=>({matches:[],tested:0,total:0,partial:false,model:"test",cache:{hits:0,misses:0}})};
  });
  try {
    const sentences = [...p.document.querySelectorAll(".review-sentence")];
    assert.equal(sentences.length, 2);
    assert.ok(sentences[0].querySelector("strong"));
    assert.equal(paragraph(p).textContent, "クリックで詳細を開きます。ホバーでは変えません。");
    sentences[0].click(); await p.window.happyDOM.waitUntilComplete();
    assert.equal(calls[0].quote, "クリックで詳細を開きます。");
    assert.ok(p.document.querySelector(".inline-peek").previousElementSibling.classList.contains("comment-anchor"));
    key(p, sentences[0], "Enter"); inputComment(p,"First comment"); click(p.document,".inline-thread button","コメントを追加");
    sentences[1].click(); await p.window.happyDOM.waitUntilComplete();
    assert.equal(calls[1].quote, "ホバーでは変えません。");
    key(p, sentences[1], "Enter"); inputComment(p,"Second comment");
    const editors = [...p.document.querySelectorAll(".inline-thread")];
    editors.find((node)=>node.querySelector("textarea")).querySelector(".button.primary").click();
    const report = await exportedNotes(p);
    assert.deepEqual(report.notes.map((note)=>note.reference.quote), ["クリックで詳細を開きます。", "ホバーでは変えません。"]);
    sentences[0].focus(); key(p,sentences[0],"j"); assert.equal(p.document.activeElement,sentences[1]);
    assert.deepEqual(p.errors,[]);
  } finally { await p.close(); }
});

test("related reading omits uncertain matches and keeps only clear correspondence", async () => {
  const input = structuredClone(snapshot); input.live = true; input.items = [];
  const p = await page(input, async () => ({ok:true,json:async()=>({matches:[
    {path:"docs/design.md",line:1,side:"new",text:"Uncertain same topic",probability:.79},
    {path:"docs/design.md",line:2,side:"new",text:"Concrete mechanism",probability:.9},
  ],tested:2,total:2,partial:false,model:"test",cache:{hits:0,misses:1}})}));
  try {
    paragraph(p).click(); await p.window.happyDOM.waitUntilComplete();
    const related = p.document.querySelector(".live-related");
    assert.ok(related.textContent.includes("Concrete mechanism"));
    assert.ok(!related.textContent.includes("Uncertain same topic"));
    assert.equal(related.querySelectorAll(".related-passage").length, 1);
    assert.equal(related.querySelector("details"), null);
    assert.deepEqual(p.errors, []);
  } finally { await p.close(); }
});

async function hotPage(input, fetcher) {
  const polls = [];
  const p = await page(input, fetcher, (window)=>{
    const schedule=window.setTimeout.bind(window);
    window.setTimeout=(fn,delay,...args)=>delay===1000 && !String(fn).includes("revokeObjectURL") ? polls.push(fn) : schedule(fn,delay,...args);
  });
  p.refresh=async()=>{assert.ok(polls.length);await polls.shift()();await p.window.happyDOM.waitUntilComplete();};
  return p;
}
function hotSnapshot() {
  const input=structuredClone(snapshot);input.live=true;input.hot=true;input.revision="before";input.items=[];
  return input;
}
function revised(input, text="Updated requirement.") {
  const next=structuredClone(input);next.id="UPDATED";next.revision="after";next.captured_at++;
  next.files=next.files.filter((file)=>file.role!=="implementation");
  next.files[0].new=`# Intent\n\n${text}\n`;
  next.files[0].new_html=`<h1 data-line="1">Intent</h1><p data-line="3">${text}</p>`;
  return next;
}
test("hot updates replace text without navigation, retain comments and drafts, and delay replacement during editing", async()=>{
  const input=hotSnapshot();const next=revised(input);let current=input;
  const p=await hotPage(input,async(url,options)=>{
    if(url==="/api/refresh") return {ok:true,json:async()=>({data:current===input?null:next,revision:current.revision,error:null})};
    return {ok:true,json:async()=>({matches:[],tested:0,total:0,partial:false,model:"test",cache:{hits:0,misses:0},revision:JSON.parse(options.body).revision})};
  });
  try {
    const sentence=p.document.querySelector('.review-sentence[data-line="3"]');
    sentence.click();await p.window.happyDOM.waitUntilComplete();
    key(p,sentence,"P");assert.equal(p.document.querySelector("textarea").value,"P");
    inputComment(p,"Keep my draft.");
    current=next;await p.refresh();
    assert.ok(paragraph(p).textContent.includes("Keep intent"));
    assert.equal(p.document.querySelector("textarea").value,"Keep my draft.");
    assert.ok(p.document.querySelector(".update-status").textContent.includes("入力を終えると反映"));
    p.document.querySelector("textarea").blur();await p.window.happyDOM.waitUntilComplete();
    assert.equal(paragraph(p).textContent,"Updated requirement.");
    assert.equal(p.document.querySelector(".previous-comments textarea").value,"Keep my draft.");
    click(p.document,".previous-comments button","コメントを追加");
    const report=await exportedNotes(p);
    assert.equal(report.snapshot.id,"UPDATED");
    assert.equal(report.notes[0].snapshot.id,input.id);
    assert.equal(report.notes[0].reference.quote,"Keep intent.");
    assert.equal(report.notes[0].text,"Keep my draft.");
    assert.deepEqual(p.errors,[]);
  } finally {await p.close();}
});
test("hot updates move an unchanged quoted comment with its sentence, without changing its original exported target", async()=>{
  const input=hotSnapshot();const next=revised(input,"Keep intent.");
  next.files[0].new_html='<h1 data-line="1">Intent</h1><p data-line="3">An inserted paragraph.</p><p data-line="5">Keep intent.</p>';
  const p=await hotPage(input,async(url)=>({ok:true,json:async()=>url==="/api/refresh"?{data:next,revision:"after",error:null}:{matches:[],tested:0,total:0,partial:false,model:"test",cache:{hits:0,misses:0}}}));
  try {
    const sentence=p.document.querySelector('.review-sentence[data-line="3"]');sentence.click();await p.window.happyDOM.waitUntilComplete();
    key(p,sentence,"Enter");inputComment(p,"A saved comment.");click(p.document,".inline-thread button","コメントを追加");
    await p.refresh();
    assert.ok(paragraph(p,5).nextElementSibling.textContent.includes("A saved comment."));
    assert.equal(p.document.querySelector(".previous-comments"),null);
    const report=await exportedNotes(p);assert.equal(report.notes[0].reference.line,3);assert.equal(report.notes[0].snapshot.id,input.id);
    assert.deepEqual(p.errors,[]);
  } finally {await p.close();}
});
test("a delayed old scan cannot populate the new comparison; a requested scan restarts against current inputs", async()=>{
  const input=hotSnapshot();input.code_scope={files:[{path:"source.txt",regions:1}],total:1,specifications:[],partial:false};
  const next=revised(input);let finish;const revisions=[];
  const p=await hotPage(input,async(url,options)=>{
    const body=JSON.parse(options.body);
    if(url==="/api/refresh")return {ok:true,json:async()=>({data:next,revision:"after",error:null})};
    revisions.push(body.revision);
    if(body.revision==="before")return await new Promise(resolve=>{finish=resolve;});
    return {ok:true,json:async()=>({revision:"after",candidates:[],outcomes:[],scanned:1,total:1,next_cursor:null,partial:false,skipped_files:0})};
  });
  try {
    click(p.document,".actions button","仕様にない実装");click(p.document,".scan-panel button","変更したコードを調べる");
    await Promise.resolve();await Promise.resolve();await p.refresh();
    finish({ok:true,json:async()=>({revision:"before",candidates:[{text:"Old result must disappear"}],outcomes:[],scanned:1,total:1,next_cursor:null,partial:false})});
    await p.window.happyDOM.waitUntilComplete();
    assert.deepEqual(revisions,["before","after"]);
    assert.ok(!p.document.body.textContent.includes("Old result must disappear"));
    assert.ok(p.document.querySelector(".scan-progress").textContent.includes("照合完了"));
    assert.deepEqual(p.errors,[]);
  } finally {await p.close();}
});


test("the sidebar follows document headings and jumps without requesting a related judgment", async()=>{
  const input=structuredClone(snapshot);input.live=true;input.items=[];
  input.files[0].new_html='<h1 data-line="1">Behavior</h1><h2 data-line="3">Saving</h2><p data-line="5">Keep intent.</p><h3 data-line="7">Failure</h3><h2 data-line="9">Saving</h2>';
  let calls=0;const p=await page(input,async()=>{calls++;throw new Error("Outline navigation must not query Jev");});
  try {
    const rows=[...p.document.querySelectorAll(".outline-row")];
    assert.deepEqual(rows.map(row=>row.textContent),["Behavior","Saving","Failure","Saving"]);
    assert.ok(Number.parseInt(rows[2].style.paddingLeft)>Number.parseInt(rows[1].style.paddingLeft));
    assert.equal(p.document.querySelector(".file-row"),null);
    assert.ok(p.document.querySelector('[aria-label="Requirements ファイル"]'));
    rows[3].click();await p.window.happyDOM.waitUntilComplete();
    assert.equal(p.document.activeElement.dataset.line,"9");
    assert.equal(calls,0);assert.equal(p.document.querySelector(".explanation-popover"),null);
    click(p.document,".navigation .stack-tab","Design");await p.window.happyDOM.waitUntilComplete();
    assert.ok(p.document.querySelector(".outline-context").textContent.includes("Design"));
    assert.equal(p.document.querySelectorAll(".outline-row").length,0);
    assert.ok(p.document.querySelector(".outline").textContent.includes("見出しはありません"));
    click(p.document,".navigation .stack-tab","Requirements");await p.window.happyDOM.waitUntilComplete();
    const search=p.document.querySelector(".sidebar .search");search.value="Failure";search.dispatchEvent(new p.window.Event("input"));
    assert.equal(p.document.querySelectorAll(".outline-row").length,1);
    assert.equal(p.document.querySelector(".outline-row").textContent,"Failure");
    assert.deepEqual(p.errors,[]);
  }finally {await p.close();}
});
test("scrolling updates the current section in the outline",async()=>{
  const input=structuredClone(snapshot);input.items=[];
  input.files[0].new_html='<h1 data-line="1">Start</h1><h2 data-line="3">Next section</h2><p data-line="5">Read here.</p>';
  const p=await page(input);
  try {
    const headings=[...p.document.querySelectorAll(".pane.current .markdown h1,.pane.current .markdown h2")];
    headings[0].getBoundingClientRect=()=>({top:-300});
    headings[1].getBoundingClientRect=()=>({top:20});
    p.window.dispatchEvent(new p.window.Event("scroll"));await p.window.happyDOM.waitUntilComplete();
    assert.equal(p.document.querySelector('.outline-row[aria-current="location"]').textContent,"Next section");
    assert.deepEqual(p.errors,[]);
  }finally {await p.close();}
});


test("copy notes writes a contextual Markdown report and keeps the original comments",async()=>{
  let copied=null;const p=await page(snapshot,null,(window)=>{window.navigator.clipboard.writeText=async(text)=>{copied=text;};});
  try {
    const text=paragraph(p);text.click();key(p,text,"Enter");inputComment(p,"Explain the concrete export method.");click(p.document,".inline-thread button","コメントを追加");
    click(p.document,".actions button","メモをコピー");await p.window.happyDOM.waitUntilComplete();
    assert.ok(copied.includes("docs/requirements.md:3 (new)"));
    assert.ok(copied.includes("Explain the concrete export method."));assert.ok(copied.includes(`Snapshot: ${snapshot.id}`));
    assert.ok(p.document.querySelector(".notes-feedback").textContent.includes("コピーしました"));
    assert.ok(p.document.querySelector(".inline-thread .note"));assert.equal(p.document.querySelector(".actions").textContent.includes("Agentに渡す"),false);
    assert.deepEqual(p.errors,[]);
  }finally {await p.close();}
});
test("a rejected clipboard write offers file export and retains the unsent warning",async()=>{
  const p=await page(snapshot,null,(window)=>{window.navigator.clipboard.writeText=async()=>{throw new Error("Permission denied");};});
  try {
    const text=paragraph(p);text.click();key(p,text,"Enter");inputComment(p,"Keep this comment.");click(p.document,".inline-thread button","コメントを追加");
    click(p.document,".actions button","メモをコピー");await p.window.happyDOM.waitUntilComplete();
    assert.ok(p.document.querySelector(".notes-feedback").textContent.includes("コピーできませんでした"));
    const unload=new p.window.Event("beforeunload",{cancelable:true});p.window.dispatchEvent(unload);assert.ok(unload.defaultPrevented);
    assert.equal((await exportedNotes(p)).notes[0].text,"Keep this comment.");assert.deepEqual(p.errors,[]);
  }finally {await p.close();}
});
test("server handoff sends the contextual report explicitly and retries with the same identity",async()=>{
  const input=structuredClone(snapshot);input.server=true;let succeed=false;const submissions=[];
  const p=await hotPage(input,async(url,options)=>{
    assert.equal(url,"/api/feedback");const submission=JSON.parse(options.body);submissions.push(submission);
    return {ok:succeed,json:async()=>succeed?{delivered:true,id:submission.id}:{error:"busy"}};
  });
  try {
    const text=paragraph(p);text.click();key(p,text,"Enter");inputComment(p,"A human review comment.");click(p.document,".inline-thread button","コメントを追加");
    assert.equal(submissions.length,0);
    click(p.document,".actions button","Agentに渡す");await p.window.happyDOM.waitUntilComplete();
    assert.ok(p.document.querySelector(".notes-feedback").textContent.includes("渡せませんでした"));
    succeed=true;click(p.document,".actions button","Agentに渡す");await p.window.happyDOM.waitUntilComplete();
    assert.equal(submissions[0].id,submissions[1].id);
    assert.equal(submissions[1].report.snapshot.id,snapshot.id);assert.equal(submissions[1].report.notes[0].text,"A human review comment.");
    assert.ok(p.document.querySelector(".notes-feedback").textContent.includes("起動元の実行出力"));assert.deepEqual(p.errors,[]);
  }finally {await p.close();}
});

test("agent-confirmed resolutions retain the original note and verified revision after updates", async()=>{
  const input=hotSnapshot();input.server=true;let resolutions=[];
  const next=structuredClone(input);next.id="updated-snapshot";next.revision="after";
  next.files[0].new="# Intent\n\nChanged text.\n";next.files[0].new_html='<h1 data-line="1">Intent</h1><p data-line="3" tabindex="0">Changed text.</p>';
  let current=input;
  const p=await hotPage(input,async(url)=>({ok:true,json:async()=>url==="/api/refresh"?{revision:current.revision,data:current===input?null:next,resolutions,error:null}:{matches:[],tested:0,total:0,partial:false,cache:{hits:0,misses:0}}}));
  try{
    key(p,paragraph(p),"Enter");inputComment(p,"Define sentence boundaries.");click(p.document,".inline-thread button","コメントを追加");
    const original=(await exportedNotes(p)).notes[0];assert.ok(original.id);
    current=next;await p.refresh();
    assert.ok(p.document.querySelector(".previous-comments").textContent.includes("変更あり・未確認"));
    assert.equal((await exportedNotes(p)).notes[0].resolution,undefined);
    key(p,paragraph(p),"Enter");const editing=inputComment(p,"Keep this unfinished draft.");
    resolutions=[{id:original.id,summary:"Verified Intl.Segmenter in Design.",snapshot:{id:next.id,base:next.base,target:next.target,captured_at:2,revision:"after"}}];
    await p.refresh();
    assert.equal(p.document.querySelector("textarea"),editing);assert.equal(editing.value,"Keep this unfinished draft.");
    editing.blur();await p.refresh();
    const solved=p.document.querySelector(".resolved-note");assert.ok(solved);assert.equal(solved.open,false);
    assert.ok(solved.querySelector("summary").textContent.includes("解決済み"));
    assert.ok(solved.textContent.includes(next.id));assert.ok(solved.textContent.includes("Verified Intl.Segmenter"));
    const note=(await exportedNotes(p)).notes[0];assert.equal(note.id,original.id);assert.deepEqual(note.reference,original.reference);assert.deepEqual(note.snapshot,original.snapshot);assert.equal(note.resolution.snapshot.revision,"after");
    const md=await exportedNotes(p,"Markdown");assert.ok(md.includes("Status: resolved"));assert.ok(md.includes("Resolved in: updated-snapshot"));assert.deepEqual(p.errors,[]);
  }finally{await p.close();}
});

test("comparison reading keeps independent pane positions and directs J/K to the pane being read", async()=>{
  const p=await page();
  const scrollFor=role=>p.document.querySelector(`.pane[data-role="${role}"] .pane-scroll`);
  try{
    click(p.document,".actions button","仕様を並べる");await p.window.happyDOM.waitUntilComplete();
    const req=scrollFor("requirements"),des=scrollFor("design");assert.ok(req&&des);
    req.scrollTop=140;req.dispatchEvent(new p.window.Event("scroll"));
    des.scrollTop=360;des.dispatchEvent(new p.window.Event("scroll"));
    assert.equal(req.scrollTop,140);
    paragraph(p).focus();des.dispatchEvent(new p.window.WheelEvent("wheel",{deltaY:80,bubbles:true}));
    assert.equal(p.document.querySelector(".pane.current").dataset.role,"design");
    key(p,p.document.activeElement,"j");assert.equal(p.document.activeElement.closest(".pane").dataset.role,"design");
    click(p.document,".actions button","仕様を並べる");await p.window.happyDOM.waitUntilComplete();
    assert.equal(scrollFor("design").scrollTop,0,"Single-document reading has no inner scroll offset");
    des.scrollTop=0;des.dispatchEvent(new p.window.Event("scroll"));
    click(p.document,".actions button","仕様を並べる");await p.window.happyDOM.waitUntilComplete();
    assert.equal(scrollFor("requirements").scrollTop,140);assert.equal(scrollFor("design").scrollTop,360);
    assert.deepEqual(p.errors,[]);
  }finally{await p.close();}
});
