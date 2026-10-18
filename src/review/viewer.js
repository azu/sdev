import { FileDiff, parseDiffFromFile } from "@pierre/diffs";
import backIcon from "./assets/back.svg";
import forwardIcon from "./assets/forward.svg";
import documentIcon from "./assets/document.svg";

const data = JSON.parse(document.getElementById("review-data").textContent);
const ja = data.language === "ja";
const t = (en, japanese) => ja ? japanese : en;
const roles = ["requirements", "design"];
// The renderer projects captured data to specifications; also filter legacy fixtures.
data.files = data.files.filter((file) => roles.includes(file.role));
for (const item of data.items) {
  item.refs = item.refs.filter((ref) => data.files.some((file) => file.path === ref.path));
  item.links = (item.links ?? []).filter((link) => item.refs.some((ref) => ref.id === link.from) && item.refs.some((ref) => ref.id === link.to));
}
const labels = { requirements: "Requirements", design: "Design" };
const fileMap = new Map(data.files.map((f) => [f.path, f]));
const app = document.getElementById("app");
const notes = [];
let exported = true;
let notesMessage = "";
let feedbackAttempt = null;
const drafts = new Map();
let commentTarget = null;
let filter = "";
let renderers = [];
let history = [];
let future = [];
let preview = null;
let relatedTimer = null;
const relatedCache = new Map();
let serverQueue = Promise.resolve();
let pendingUpdate = null;
let updateMessage = "";
let composing = false;
let pendingResolutionRender = false;
let scan = { stopping:false, recent:[], withoutCandidate:0, running: false, scanned: 0, total: null, cursor: 0, candidates: [], error: null, partial: false, skipped: 0 };
let state = { active: "requirements", paths: {}, sides: {}, modes: {}, scrolls: {}, item: null, reference: null, overview: false, split: false, detail: false, pageScroll: 0, scan: false, readingMode: "inline" };

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}
function button(text, action, className = "button") {
  const node = el("button", className, text);
  node.type = "button";
  node.onclick = action;
  return node;
}
function iconButton(asset, label, action) {
  const node = button(null, action, "icon-button");
  node.title = label;
  node.setAttribute("aria-label", label);
  const image = el("img"); image.src = asset; image.width = 20; image.height = 20; image.alt = "";
  node.append(image);
  return node;
}
function badge(text, kind = "neutral") { return el("span", `badge ${kind}`, text); }
function currentFile(role) { return fileMap.get(state.paths[role]); }
function item() { return data.items.find((i) => i.id === state.item); }
function referenceName(ref) { return ref.label || fileMap.get(ref.path)[ref.side]?.split("\n")[ref.line - 1]?.trim() || ref.path; }
function sameReference(a, b) { return a?.path === b?.path && a?.line === b?.line && a?.side === b?.side; }
function key(role) { return [role, state.paths[role], state.sides[role], state.modes[role]].join(":"); }
function rememberPagePosition() {
  state.pageScroll = window.scrollY;
  if (!state.overview) return;
  for (const pane of document.querySelectorAll(".pane[data-role]")) {
    const scroll = pane.querySelector(".pane-scroll");
    if (scroll) state.scrolls[key(pane.dataset.role)] = scroll.scrollTop;
  }
}
function go(change) {
  rememberPagePosition();
  history.push(structuredClone(state)); future = [];
  change(); render();
}
function pickReference(ref) {
  const role = fileMap.get(ref.path).role;
  go(() => { state.paths[role] = ref.path; state.sides[role] = ref.side; state.active = role; state.reference = ref; state.detail = true; });
  requestAnimationFrame(() => { highlight(ref, true); focusReference(ref); });
}
function pickItem(selected, remember = true) {
  const change = () => {
    state.item = selected.id; state.detail = true; state.pageScroll = 0;
    for (const role of roles) {
      const ref = selected.refs.find((r) => fileMap.get(r.path).role === role);
      if (ref) { state.paths[role] = ref.path; state.sides[role] = ref.side; }
    }
    const first = selected.refs.find((r) => fileMap.get(r.path).role === "requirements") ?? selected.refs[0];
    state.reference = first;
    state.active = first ? fileMap.get(first.path).role : "requirements";
  };
  if (remember) go(change); else change();
  if (remember) requestAnimationFrame(() => {
    highlight(state.reference, true);
    if (state.reference) focusReference(state.reference);
    else document.querySelector(`[data-unanchored-item="${CSS.escape(selected.id)}"]`)?.focus();
  });
}

function sourceOwner(blocks, line, quote = null) {
  if (quote) {
    const sentence = blocks.find((block) => Number(block.dataset.line) === line && block.dataset.quote === quote);
    if (sentence) return sentence;
  }
  return blocks.filter((block) => Number(block.dataset.line) <= line && Number(block.dataset.lineEnd ?? block.dataset.line) >= line)
    .sort((a, b) => (Number(a.dataset.lineEnd ?? a.dataset.line) - Number(a.dataset.line)) - (Number(b.dataset.lineEnd ?? b.dataset.line) - Number(b.dataset.line))
      || (a.contains(b) ? 1 : -1))[0];
}
function entriesAt(path, line, side) {
  return data.items.flatMap((selected) => selected.refs.filter((ref) => ref.path === path && ref.side === side && ref.line === line).map((ref) => ({ selected, ref })));
}
function positionPreview() {
  if (!preview) return;
  const inline = preview.pinned && state.readingMode === "inline" && preview.trigger.closest(".markdown");
  preview.node.classList.toggle("inline-peek", Boolean(inline));
  if (inline) {
    const owner = preview.trigger.closest("table") ?? (preview.trigger.classList.contains("review-sentence") ? preview.trigger.parentElement.closest("[data-line]") : preview.trigger);
    const after = owner.nextElementSibling;
    if (after?.classList.contains("comment-anchor")) after.after(preview.node);
    else owner.after(preview.node);
    preview.node.style.cssText = ""; return;
  }
  if (preview.node.parentElement !== document.body) document.body.append(preview.node);
  const rect = preview.trigger.getBoundingClientRect();
  const width = Math.min(560, window.innerWidth - 32);
  const left = Math.max(16, Math.min(rect.left, window.innerWidth - width - 16));
  const top = rect.bottom + 8;
  const below = window.innerHeight - top - 16;
  const above = rect.top - 24;
  const useBelow = below >= Math.min(280, above);
  const height = Math.max(120, Math.min(window.innerHeight * .7, useBelow ? below : above));
  preview.node.style.cssText = `width:${width}px;max-height:${height}px;left:${left}px;${useBelow ? `top:${Math.max(16, top)}px` : `bottom:${Math.max(16, window.innerHeight - rect.top + 8)}px`}`;
}
function closePreview(restoreFocus = false) {
  clearTimeout(relatedTimer);
  if (!preview) return;
  const previous = preview; preview = null;
  previous.trigger.classList.remove("review-focus");
  previous.trigger.querySelectorAll(".review-marker").forEach((node) => node.setAttribute("aria-expanded", "false"));
  previous.node.remove();
  if (restoreFocus) previous.trigger.focus({ preventScroll: true });
}
function showPreview(entries, trigger, pinned = false, section = false) {
  if (preview?.trigger === trigger) { preview.pinned ||= pinned; positionPreview(); return; }
  closePreview();
  const popup = el("section", "explanation-popover"); popup.id = "explanation-popover";
  popup.setAttribute("role", "dialog"); popup.setAttribute("aria-label", t("Details for this selection", "この箇所の詳細"));
  const bar = el("div", "popover-heading");
  if (data.live) popup.classList.add("live-peek");
  else bar.append(el("strong", null, section ? t("Material in this section", "この節の判断材料") : t("Understand this passage", "この文章を具体的に見る")));
  bar.append(button("×", () => closePreview(true), "popover-close"));
  bar.lastChild.setAttribute("aria-label", t("Close details", "詳細を閉じる"));
  popup.append(bar);
  if (!data.live && !entries.length) popup.append(el("p", "muted", t("No agent explanation for this passage yet. Type or press Enter to comment on it.", "この文章へのAgentの説明はまだありません。文字入力またはEnterでコメントできます。")));
  for (const { selected, ref } of entries) {
    const entry = el("section", "passage-explanation"); entry.dataset.item = selected.id;
    entry.append(el("h3", null, selected.title), renderMaterial(selected, ref));
    const connections = (selected.links ?? []).filter((link) => link.from === ref.id || link.to === ref.id);
    for (const link of connections) {
      const other = selected.refs.find((r) => r.id === (link.from === ref.id ? link.to : link.from));
      const connection = el("div", "passage-connection");
      connection.append(el("p", null, link.reason), button(`${labels[fileMap.get(other.path).role]} · ${referenceName(other)} ↗`, () => pickReference(other), "trace-endpoint"));
      entry.append(connection);
    }
    if (!connections.length) entry.append(el("p", "trace-missing-links", t("No connection reason supplied; this alone does not establish a problem.", "接続理由は未記述です。それだけで仕様の問題とは判断できません。")));
    popup.append(entry);
  }
  popup.onclick = (event) => event.stopPropagation();
  preview = { node: popup, trigger, entries, pinned };
  trigger.classList.add("review-focus");
  trigger.querySelectorAll(".review-marker").forEach((node) => node.setAttribute("aria-expanded", "true"));
  document.body.append(popup); positionPreview(); renderComments(false);
  if (data.live) {
    const card = trigger.closest(".document-card");
    const path = card?.dataset.path;
    const line = Number(trigger.dataset.line);
    if (path && line) {
      const side = trigger.dataset.side ?? state.sides[fileMap.get(path).role];
      const target = el("div", "live-related"); target.append(el("p", "muted", t("Finding related passages…", "関連する箇所を調べています…"))); popup.append(target);
      const reference = { path, line, side };
      if (trigger.dataset.quote) reference.quote = trigger.dataset.quote;
      relatedTimer = setTimeout(() => loadRelated(reference, target), 180);
    }
  }
}
function mountPassageDetails(content, file, side) {
  const blocks = [...content.querySelectorAll("[data-line]")];
  const matching = data.items.flatMap((selected) => selected.refs.filter((ref) => ref.path === file.path && ref.side === side).map((ref) => ({ selected, ref })));
  for (const block of blocks) {
    const direct = matching.filter(({ ref }) => sourceOwner(blocks, ref.line) === block);
    let entries = direct;
    const level = /^H[1-6]$/.test(block.tagName) ? Number(block.tagName[1]) : null;
    if (level && !direct.length) {
      const start = Number(block.dataset.line);
      const next = blocks.find((candidate) => /^H[1-6]$/.test(candidate.tagName) && Number(candidate.tagName[1]) <= level && Number(candidate.dataset.line) > start);
      entries = matching.filter(({ ref }) => ref.line > start && ref.line < Number(next?.dataset.line ?? Infinity));
    }
    entries = [...new Map(entries.map((entry) => [entry.selected.id, entry])).values()];
    block.reviewEntries = entries;
    block.reviewSection = !direct.length;
    if (data.live) continue;
    if (!entries.length && !data.live) continue;
    block.classList.add("review-linked");
    const marker = button(data.live ? t("Related", "関連") : `${t("Examples / reasons", "具体例・理由")} ${entries.length}`, (event) => {
      event.stopPropagation(); block.focus({ preventScroll: true });
      showPreview(entries, block, true, !direct.length);
    }, "review-marker");
    marker.setAttribute("aria-haspopup", "dialog"); marker.setAttribute("aria-expanded", "false"); marker.setAttribute("aria-controls", "explanation-popover");
    marker.onkeydown = (event) => {
      if (event.key === "Enter" || event.key === " ") return;
      startTypingComment(event, { reference: { path: file.path, line: Number(block.dataset.line), side }, item: entries.length === 1 ? entries[0].selected.id : null }, block.nextElementSibling);
    };
    if (block.tagName === "TR") block.cells[0]?.append(marker);
    else block.append(marker);
  }
}
function wrapSentences(content) {
  if (!data.live || typeof Intl.Segmenter !== "function") return;
  const segmenter = new Intl.Segmenter(data.language, { granularity: "sentence" });
  for (const block of [...content.querySelectorAll("p[data-line],li[data-line]")].filter((node) => !node.querySelector("[data-line]"))) {
    const text = block.textContent;
    const segments = [...segmenter.segment(text)];
    for (const { segment, index } of segments.reverse()) {
      const quote = segment.replace(/\s+/gu, " ").trim(); if (!quote) continue;
      const walker = document.createTreeWalker(block, NodeFilter.SHOW_TEXT);
      const nodes = []; let offset = 0;
      while (walker.nextNode()) { const node = walker.currentNode; nodes.push({ node, start:offset, end:offset + node.length }); offset += node.length; }
      const start = nodes.find((entry) => entry.end > index);
      const end = nodes.find((entry) => entry.end >= index + segment.length);
      if (!start || !end) continue;
      const range = document.createRange();
      range.setStart(start.node, index - start.start); range.setEnd(end.node, index + segment.length - end.start);
      const sentence = el("span", "review-sentence");
      sentence.tabIndex = 0; sentence.dataset.line = block.dataset.line;
      sentence.dataset.lineEnd = block.dataset.lineEnd ?? block.dataset.line;
      sentence.dataset.quote = quote;
      sentence.append(range.extractContents()); range.insertNode(sentence);
    }
  }
}
function prepareTableRows(content) {
  for (const row of content.querySelectorAll("table tr[data-line]")) {
    row.dataset.quote = [...row.cells].map((cell) => cell.textContent.replace(/\s+/gu, " ").trim()).join(" | ");
  }
}
function focusReference(ref) {
  const card = [...document.querySelectorAll(".document-card")].find((node) => node.dataset.path === ref.path);
  const block = sourceOwner([...card?.querySelectorAll(".markdown [data-line]") ?? []], ref.line, ref.quote);
  if (block) { block.focus({ preventScroll: true }); showPreview(block.reviewEntries ?? [], block, true, block.reviewSection); }
}
function selectLine(file, line, side, origin, quote = null) {
  state.reference = { path: file.path, line, side };
  if (quote) state.reference.quote = quote;
  highlight(state.reference, false);
  const block = origin?.reviewBlock;
  if (block) {
    block.focus({ preventScroll: true }); showPreview(block.reviewEntries ?? [], block, true, block.reviewSection);
  } else {
    // A focusable action is mounted as an inline annotation for a selected diff line.
    renderComments();
    const action = origin?.querySelector(`.diff-line-focus[data-line="${line}"][data-side="${side}"]`);
    action?.focus();
    if (action) queueMicrotask(() => { if (action.isConnected) showPreview(entriesAt(file.path, line, side), action, true); });
  }
}
function startTypingComment(event, target, origin) {
  if (["j", "k"].includes(event.key.toLowerCase()) || event.metaKey || event.ctrlKey || event.altKey || event.target.closest("textarea, input, select, .inline-thread, .explanation-popover")) return;
  const composing = event.key === "Process" || event.keyCode === 229 || event.isComposing;
  if (event.key !== "Enter" && event.key.length !== 1 && !composing) return;
  if (!composing) event.preventDefault();
  closePreview();
  if (data.hot && !target.snapshot) target = {...target,snapshot:comparisonIdentity()};
  const key = targetKey(target);
  if (event.key !== "Enter" && !composing) drafts.set(key, (drafts.get(key) ?? "") + event.key);
  openComment(target, origin);
}
function highlight(ref, scroll) {
  document.querySelectorAll(".selected-line").forEach((n) => n.classList.remove("selected-line"));
  if (!ref) return;
  const card = [...document.querySelectorAll(".document-card")].find((c) => c.dataset.path === ref.path);
  if (!card) return;
  const blocks = [...card.querySelectorAll(".markdown [data-line]")];
  const block = sourceOwner(blocks, ref.line, ref.quote);
  if (block) {
    block.classList.add("selected-line");
    if (scroll) block.scrollIntoView({ block: "center", inline: "nearest" });
  }
  const renderer = renderers.find((r) => r.path === ref.path);
  if (renderer) {
    const side = ref.side === "old" ? "deletions" : "additions";
    renderer.instance.setSelectedLines({ start: ref.line, end: ref.line, side });
    // Pierre renders a Shadow DOM. Navigate the actual rendered line after mount.
    if (scroll) requestAnimationFrame(() => {
      const shadow = card.querySelector("diffs-container")?.shadowRoot;
      const line = shadow?.querySelector(`[data-line="${ref.line}"][data-line-type="${side === "deletions" ? "change-deletion" : "change-addition"}"]`)
        ?? shadow?.querySelector(`[data-line="${ref.line}"]`);
      line?.scrollIntoView({ block: "center", inline: "nearest" });
    });
  }
}
function preferredFiles(role) {
  return data.files.filter((f) => f.role === role).sort((a, b) => {
    const score = (f) => (f.path.endsWith(".ja.md") === ja ? 0 : 1) + (f.changed ? 0 : 2);
    return score(a) - score(b) || a.path.localeCompare(b.path);
  });
}
for (const role of roles) {
  const file = preferredFiles(role)[0];
  if (file) { state.paths[role] = file.path; state.sides[role] = file.new != null ? "new" : "old"; }
  state.modes[role] = "read";
}
if (data.items.length) pickItem(data.items[0], false);
else state.active = roles.find((r) => currentFile(r)) ?? "requirements";

function outlineEntries() {
  const file = currentFile(state.active);
  const side = state.sides[state.active];
  const source = el("div"); source.innerHTML = file?.[`${side}_html`] ?? "";
  return [...source.querySelectorAll("h1[data-line],h2[data-line],h3[data-line],h4[data-line],h5[data-line],h6[data-line]")]
    .map((heading)=>({text:heading.textContent,line:Number(heading.dataset.line),level:Number(heading.tagName[1])}));
}
function jumpToHeading(entry) {
  const role=state.active;
  go(()=>{state.scan=false;state.modes[role]="read";state.reference=null;});
  requestAnimationFrame(()=>{
    const heading=document.querySelector(`.pane.current .markdown h${entry.level}[data-line="${entry.line}"]`);
    if (!heading) return;
    heading.focus({preventScroll:true});
    heading.scrollIntoView({block:"start",behavior:"instant"});
    updateOutlinePosition();
  });
}
function renderOutline(container) {
  container.replaceChildren();
  const entries=outlineEntries();
  const query=filter.toLocaleLowerCase();
  const visible=entries.filter((entry)=>entry.text.toLocaleLowerCase().includes(query));
  const baseLevel=entries.length ? Math.min(...entries.map((entry)=>entry.level)) : 1;
  for (const entry of visible) {
    const row=button(entry.text,()=>jumpToHeading(entry),"outline-row");
    row.dataset.line=entry.line;
    row.style.paddingLeft=`${10+Math.min(entry.level-baseLevel,4)*12}px`;
    container.append(row);
  }
  if (!visible.length) container.append(el("p","empty-small",entries.length ? t("No matching headings.","該当する見出しがありません。") : t("This document has no headings.","この文書に見出しはありません。")));
}
let outlineFrame = null;
function updateOutlinePosition() {
  const headings=[...document.querySelectorAll(".pane.current .markdown h1[data-line],.pane.current .markdown h2[data-line],.pane.current .markdown h3[data-line],.pane.current .markdown h4[data-line],.pane.current .markdown h5[data-line],.pane.current .markdown h6[data-line]")];
  const top=state.overview ? (document.querySelector(".pane.current .pane-scroll")?.getBoundingClientRect().top ?? 0)+16
    : (document.querySelector(".topbar")?.getBoundingClientRect().bottom ?? 0)+32;
  const current=headings.filter((heading)=>heading.getBoundingClientRect().top<=top).at(-1) ?? headings[0];
  for (const row of document.querySelectorAll(".outline-row")) {
    const active=Boolean(current && row.dataset.line===current.dataset.line && !state.scan);
    row.classList.toggle("active",active);
    if(active) row.setAttribute("aria-current","location");else row.removeAttribute("aria-current");
  }
}
function scheduleOutlinePosition() {
  if (outlineFrame != null) return;
  outlineFrame=requestAnimationFrame(()=>{outlineFrame=null;updateOutlinePosition();});
}
function activateReadingPane(pane) {
  const role=pane.dataset.role;
  if (!role || role===state.active) return;
  state.active=role;
  document.querySelectorAll(".pane").forEach((node)=>node.classList.toggle("current",node.dataset.role===role));
  document.querySelectorAll(".stack-tab").forEach((node)=>node.classList.toggle("active",node.dataset.role===role));
  document.querySelector(".sidebar")?.replaceWith(renderSidebar());
  scheduleOutlinePosition();
}
function renderSidebar() {
  const sidebar = el("aside", "sidebar");
  const brand = el("div", "brand", "sdev"); brand.append(el("span", "brand-sub", "review"));
  sidebar.append(brand, el("div", "sidebar-label", t("CONTENTS", "目次")),
    el("div","outline-context",`${labels[state.active]} · ${state.sides[state.active]==="old" ? t("Before","変更前") : t("After","変更後")}`));
  const search=el("input","search");search.type="search";
  search.placeholder=t("Find a heading…","見出しを探す…");search.value=filter;
  search.setAttribute("aria-label",search.placeholder);
  const list=el("nav","sidebar-list outline");list.setAttribute("aria-label",t("Document contents","文書の目次"));
  search.oninput=()=>{filter=search.value;renderOutline(list);updateOutlinePosition();};
  sidebar.append(search,list);renderOutline(list);
  if (data.items.length) {
    const extra=el("details","agent-items");extra.append(el("summary",null,t(`Agent material · ${data.items.length}`,`Agentの補助項目 · ${data.items.length}`)));
    for (const selected of data.items) extra.append(button(selected.title,()=>pickItem(selected),`review-item ${state.item===selected.id ? "active" : ""}`));
    sidebar.append(extra);
  }
  if (data.live) {
    const entry=button(null,()=>go(()=>{state.scan=true;state.pageScroll=0;}),`review-item scan-entry ${state.scan ? "active" : ""}`);
    entry.append(el("span","item-title",t("Implementation without a specification","仕様にない実装")),el("span","scan-status",scan.total===null?t("Not checked","未検査"):`${scan.candidates.length} ${t("candidates","件の候補")} · ${scan.scanned}/${scan.total}`));
    sidebar.append(entry);
  }
  const footer=el("div","sidebar-footer");
  footer.append(badge(t("LOCAL SNAPSHOT","ローカルの記録")),el("p",null,data.live?t("Jev connected · No repository edits","Jev接続あり · リポジトリ編集なし"):t("No network · No repository edits","通信なし · リポジトリ編集なし")));
  if(!data.live && !data.items.length) footer.append(el("p",null,t("No agent analysis yet.","Agentの分析はまだありません。")));
  sidebar.append(footer);return sidebar;
}

function renderHeader() {
  const header = el("header", "topbar");
  const navigation = el("div", "navigation");
  const back = iconButton(backIcon, t("Back", "戻る"), () => { if (history.length) { rememberPagePosition(); future.push(structuredClone(state)); state = history.pop(); render(); } });
  const forward = iconButton(forwardIcon, t("Forward", "進む"), () => { if (future.length) { rememberPagePosition(); history.push(structuredClone(state)); state = future.pop(); render(); } });
  back.disabled = !history.length; forward.disabled = !future.length;
  navigation.append(back, forward);
  for (const role of roles) {
    const tab=button(labels[role], () => go(() => { state.active = role; state.detail = true; state.scan = false; }), `stack-tab ${state.active === role ? "active" : ""}`);
    tab.dataset.role=role;navigation.append(tab);
    if (role !== roles.at(-1)) navigation.append(el("span", "chevron", "›"));
  }
  const actions = el("div", "actions");
  const readingMode = el("select", "reading-mode"); readingMode.setAttribute("aria-label", t("Related view", "関連の表示方法"));
  for (const [value, label] of [["inline", t("Inline peek", "本文内に開く")], ["popover", t("Popover", "ポップオーバー")]]) {
    const option = el("option", null, label); option.value = value; option.selected = state.readingMode === value; readingMode.append(option);
  }
  readingMode.onchange = () => { state.readingMode = readingMode.value; positionPreview(); };
  actions.append(readingMode);
  if (data.live) actions.append(button(t("Implementation without a specification", "仕様にない実装"), () => go(() => { state.scan = !state.scan; state.pageScroll = 0; }), `button ${state.scan ? "active" : ""}`));
  actions.append(button(t("Compare specifications", "仕様を並べる"), () => go(() => { state.overview = !state.overview; state.detail = true; }), `button ${state.overview ? "active" : ""}`),
    button(t("Copy notes", "メモをコピー"), () => copyNotes(), "button primary"));
  if (data.server) actions.append(button(t("Send to agent", "Agentに渡す"), () => sendNotes(), "button"));
  actions.append(button(t("Export notes", "メモを書き出す"), () => exportNotes("md"), "button"),button(t("Export JSON", "JSONを書き出す"), () => exportNotes("json"), "button"));
  const feedback=el("span","notes-feedback",notesMessage);feedback.setAttribute("role","status");actions.append(feedback);
  navigation.append(el("span", "reading-hint", t("J/K · sentences / rows   Enter · comment", "J/Kで文・行を移動 · Enterでコメント")));
  header.append(navigation, actions); return header;
}

function renderPane(role) {
  const file = currentFile(role);
  const active = state.active === role;
  const pane = el("section", `pane ${active ? "current" : ""}`);
  pane.dataset.role=role;
  if (!state.overview && !active) {
    pane.classList.add("folded");
    const fold = button(`${roles.indexOf(role) + 1}  ${labels[role]}`, () => go(() => { state.active = role; state.detail = true; }), "fold-button");
    pane.append(fold); return pane;
  }
  const bar = el("div", "pane-bar");
  bar.append(el("span", "pane-number", `0${roles.indexOf(role) + 1}`), el("strong", null, labels[role]));
  const files = preferredFiles(role);
  if (files.length) {
    const select = el("select", "file-select"); select.setAttribute("aria-label", `${labels[role]} ${t("file", "ファイル")}`);
    for (const f of files) { const option = el("option", null, f.path); option.value = f.path; option.selected = file === f; select.append(option); }
    select.onchange = () => go(() => { state.paths[role] = select.value; state.sides[role] = fileMap.get(select.value).new != null ? "new" : "old"; state.reference = null; });
    bar.append(select);
  }
  pane.append(bar);
  if (!file) {
    pane.append(el("div", "empty-state", t(`No ${labels[role]} captured. Add context with --spec PATH, or ask the agent to revisit missing specifications.`, `${labels[role]}が取り込まれていません。--spec PATHで読む材料を追加するか、Agentと仕様の不足を確認してください。`)));
    return pane;
  }
  const scroll = el("div", "pane-scroll"); const scrollKey = key(role);
  scroll.onscroll = () => {
    if (!scroll.isConnected) return;
    if (state.overview) state.scrolls[scrollKey] = scroll.scrollTop;
    scheduleOutlinePosition();
  };
  scroll.addEventListener("wheel", () => activateReadingPane(pane), {passive:true});
  scroll.addEventListener("pointerdown", () => activateReadingPane(pane));
  const card = el("article", "document-card"); card.dataset.path = file.path;
  const title = el("div", "document-heading");
  const image = el("img"); image.src = documentIcon; image.width = 12; image.height = 12; image.alt = "";
  const type = badge(labels[role]); type.prepend(image);
  title.append(type, badge(file.changed ? t("Changed", "変更あり") : t("Reading context", "読む材料"), file.changed ? "green" : "neutral"),
    el("h1", null, file.path.split("/").at(-2) || file.path), el("p", "file-path", file.path));
  const controls = el("div", "document-controls");
  if (roles.includes(role)) {
    for (const mode of ["read", "diff"]) controls.append(button(mode === "read" ? t("Document", "本文") : t("Diff", "差分"), () => go(() => { state.modes[role] = mode; }), `segmented ${state.modes[role] === mode ? "active" : ""}`));
  }
  const isDiff = state.modes[role] === "diff";
  if (isDiff) controls.append(button(state.split ? t("Unified diff", "統合差分へ") : t("Split diff", "左右差分へ"), () => go(() => { state.split = !state.split; }), "button"));
  else for (const side of ["old", "new"]) {
    const choice = button(side === "old" ? t("Before", "変更前") : t("After", "変更後"), () => go(() => { state.sides[role] = side; }), `segmented ${state.sides[role] === side ? "active" : ""}`);
    choice.disabled = file[side] == null; controls.append(choice);
  }
  card.append(title, controls);
  const refs = item()?.refs.filter((r) => r.path === file.path) ?? [];
  if (refs.length) {
    const ribbon = el("div", "reference-ribbon", t("Agent references: ", "Agentの参照箇所: "));
    for (const ref of refs) ribbon.append(button(`${ref.side === "old" ? "−" : "+"}${ref.line}`, () => pickReference(ref), "reference-link"));
    card.append(ribbon);
  }
  if (file.notice) card.append(el("pre", "warning", file.notice));
  if (isDiff && !file.notice && file.old === file.new) {
    card.append(el("div", "empty-state", t("No text differences. Git may report a change to file metadata.", "本文の差分はありません。Gitがファイルの属性の変更を検出している場合もあります。")));
  } else if (isDiff && !file.notice) {
    const container = el("diffs-container", "diff-view"); card.append(container);
    const oldFile = file.old == null ? undefined : { name: file.path, contents: file.old, lang: "text" };
    const newFile = file.new == null ? undefined : { name: file.path, contents: file.new, lang: "text" };
    if (oldFile || newFile) {
      const parsed = parseDiffFromFile(oldFile, newFile); parsed.lang = "text";
      const instance = new FileDiff({
        theme: "github-light", themeType: "light", diffStyle: state.split ? "split" : "unified", overflow: "scroll", disableFileHeader: true,
        expandUnchanged: Boolean(state.reference?.path === file.path),
        onLineClick: ({ lineNumber, annotationSide }) => selectLine(file, lineNumber, annotationSide === "deletions" ? "old" : "new", container),
        renderAnnotation: ({ lineNumber, side }) => renderDiffAnnotation(file, lineNumber, side === "deletions" ? "old" : "new"),
        onPostRender: () => { if (state.reference?.path === file.path) highlight(state.reference, false); },
      });
      renderers.push({ instance, path: file.path });
      instance.render({ fileDiff: parsed, fileContainer: container, lineAnnotations: annotations(file.path) });
    } else card.append(el("div", "empty-state", t("No text available.", "表示できる本文がありません。")));
  } else if (!isDiff) {
    const content = el("div", "markdown"); content.innerHTML = file[`${state.sides[role]}_html`] ?? ""; // Only Rust's safe Markdown renderer creates this field.
    function select(event) {
      if (event.target.closest("button, input, textarea, select, .inline-thread, .comment-anchor")) return;
      const block = event.target.closest("[data-line]");
      if (block) selectLine(file, Number(block.dataset.line), state.sides[role], { reviewBlock:block }, block.dataset.quote);
    }
    content.onclick = select;
    content.onkeydown = (event) => {
      if (event.target.closest("button, input, textarea, select, .inline-thread, .comment-anchor")) return;
      const block = event.target.closest("[data-line]"); if (!block) return;
      const ref = { path: file.path, line: Number(block.dataset.line), side: state.sides[role] };
      if (block.dataset.quote) ref.quote = block.dataset.quote;
      const selected = data.items.find((i) => i.refs.some((r) => sameReference(r, ref)));
      startTypingComment(event, { reference: ref, item: selected?.id ?? null }, content);
    };
    prepareTableRows(content);
    const tableAnchors = new Map();
    for (const block of content.querySelectorAll("[data-line]")) {
      const anchor = commentAnchor({ reference: { path: file.path, line: Number(block.dataset.line), side: state.sides[role] }, item: state.item });
      anchor.reviewEnd = Number(block.dataset.lineEnd ?? block.dataset.line);
      anchor.reviewBlock = block;
      const table = block.closest("table");
      if (table) {
        (tableAnchors.get(table) ?? table).after(anchor);
        tableAnchors.set(table, anchor);
      } else block.after(anchor);
    }
    wrapSentences(content);
    mountPassageDetails(content, file, state.sides[role]);
    card.append(content);
  }
  scroll.append(card); pane.append(scroll);
  requestAnimationFrame(() => {
    if (!scroll.isConnected) return;
    if (state.overview) scroll.scrollTop = state.scrolls[scrollKey] ?? 0;
    highlight(state.reference, false);
  });
  return pane;
}

function investigationDetails(text) {
  const details = el("details", "investigation-details");
  details.append(el("summary", null, t("Inspected scope and limits", "調べた範囲と限界")), el("p", "muted", text)); return details;
}
function renderDiffAnnotation(file, line, side) {
  const ref = { path: file.path, line, side };
  const entries = entriesAt(file.path, line, side);
  const selected = entries[0]?.selected;
  const target = { reference: ref, item: selected?.id ?? null };
  const annotation = el("div", "diff-annotation");
  const action = button(entries.length ? t("Examples / reasons", "具体例・理由") : t("Line details · type to comment", "行の詳細・入力してコメント"), () => showPreview(entries, action, true), "diff-line-focus");
  action.dataset.line = String(line); action.dataset.side = side;
  action.onkeydown = (event) => startTypingComment(event, target, annotation.closest("diffs-container") ?? annotation);
  action.setAttribute("aria-haspopup", "dialog");
  annotation.append(action);
  const thread = renderThread(target); if (thread) annotation.append(thread);
  return annotation;
}
function renderMaterial(selected, reference = null) {
  const material = el("div", "review-material");
  const kinds = { aligned: t("Consistent with specification", "仕様と対応"), missing_requirement: t("Requirement to record", "要求に残すか判断"), missing_design: t("Design decision to record", "設計に残すか判断"), mismatch: t("Contradiction to resolve", "矛盾を確認"), uncertain: t("Insufficient evidence", "根拠が不足") };
  material.append(badge(selected.attention ? t("Needs attention", "要確認") : t("Context", "判断の材料"), selected.attention ? "amber" : "blue"));
  if (selected.assessment) {
    const a = selected.assessment;
    material.append(el("h3", null, kinds[a.kind]), el("p", "assessment-summary", a.summary), el("p", null, `${t("User impact: ", "利用者への影響: ")}${a.impact}`), investigationDetails(a.inspected));
  }
  if (selected.question) material.append(el("p", "question", selected.question));
  const target = { reference, item: selected.id };
  material.append(button(t("Comment", "コメントを書く"), () => { closePreview(); openComment(target); }, "button"));
  if (!reference) material.append(commentAnchor(target));
  const examples = el("div", "examples");
  for (const e of selected.examples ?? []) {
    const section = el("section", "material");
    section.append(badge({ observed: t("Observed", "実行して観測"), illustrative: t("Illustrative", "説明用の例"), unverified: t("Unverified", "未確認") }[e.kind], e.kind === "observed" ? "green" : "amber"), el("h3", null, e.label), el("div", "material-label", t("Input", "入力")), el("pre", "example-text", e.input), el("div", "material-label", t("Result", "結果")), el("pre", "example-text", e.result));
    examples.append(section);
  }
  material.append(examples);
  for (const d of selected.decisions ?? []) {
    const section = el("section", "material");
    section.append(badge({ recorded: t("Recorded reason", "記録された理由"), inferred: t("Inferred explanation", "推定した説明"), unknown: t("Reason unknown", "理由不明") }[d.basis], "neutral"), el("h3", null, d.text), el("p", null, d.reason));
    material.append(section);
  }
  return material;
}

function targetKey(target) {
  const r = target?.reference;
  const snapshot = target?.snapshot ?? (data.hot ? comparisonIdentity() : null);
  if (snapshot) return JSON.stringify([snapshot.id, r?.path, r?.side, r?.line, r?.quote, target.item]);
  return r ? JSON.stringify([r.path, r.side, r.line, r.quote ?? null]) : JSON.stringify(["item", target?.item]);
}
function commentAnchor(target) {
  const anchor = el("div", "comment-anchor"); anchor.reviewTarget = target; return anchor;
}
function openComment(target, origin) {
  commentTarget = structuredClone(target);
  if (data.hot && !commentTarget.snapshot) commentTarget.snapshot = comparisonIdentity();
  closePreview();
  renderComments(); renderPreviousComments();
  // Keep focus at the selected passage when both an excerpt and full text exist.
  (origin?.querySelector(".note-input") ?? document.querySelector(".note-input"))?.focus();
}
function renderThread(target) {
  const key = targetKey(target);
  const saved = notes.filter((note) => targetKey(note) === key);
  const editing = commentTarget && targetKey(commentTarget) === key;
  if (!saved.length && !editing) return undefined;
  const thread = el("section", "inline-thread");
  thread.setAttribute("aria-label", t("Inline comments", "インラインコメント"));
  thread.onclick = (event) => event.stopPropagation();
  const ref = target.reference;
  thread.append(el("p", "note-target", ref ? `${ref.path}:${ref.line} (${ref.side})` : `${t("Review item", "項目")}: ${target.item}`));
  if (ref?.quote) thread.append(el("blockquote", "comment-quote", ref.quote));
  for (const note of saved) {
    const box = el(note.resolution ? "details" : "div", "note");
    if (note.resolution) {
      box.classList.add("resolved-note");
      box.append(el("summary", null, `${t("Resolved", "解決済み")} · ${note.text.split("\n")[0]}`),
        el("p", "resolution-summary", note.resolution.summary),
        el("p", "muted", `${t("Verified snapshot", "確認した版")}: ${note.resolution.snapshot.id}`));
    } else box.append(badge(note.snapshot && note.snapshot.id !== data.id && !displayReference(note) ? t("Changed · needs verification", "変更あり・未確認") : t("Open", "未解決")));
    box.append(el("p", null, note.text), button(t("Remove", "削除"), () => { notes.splice(notes.indexOf(note), 1); exported = false; renderComments(); renderPreviousComments(); }, "text-button"));
    thread.append(box);
  }
  if (editing) {
    const input = el("textarea", "note-input"); input.value = drafts.get(key) ?? "";
    input.dataset.targetKey = key;
    input.placeholder = t("Intent, question, or change needed…", "意図・問い・変更してほしいこと…");
    input.setAttribute("aria-label", t("Inline comment", "インラインコメント"));
    input.oninput = () => {
      drafts.set(key, input.value);
      // Synchronize a second visible copy without interrupting the active input.
      document.querySelectorAll(".note-input").forEach((peer) => {
        if (peer.dataset.targetKey === key && peer !== input) peer.value = input.value;
      });
    };
    thread.append(input, button(t("Add comment", "コメントを追加"), () => {
      const text = (drafts.get(key) ?? "").trim(); if (!text) return;
      notes.push({ id: crypto.randomUUID(), text, reference: commentTarget.reference ? { ...commentTarget.reference } : null, item: commentTarget.item ?? null, ...(commentTarget.snapshot ? {snapshot:commentTarget.snapshot} : {}) });
      drafts.delete(key); commentTarget = null; exported = false; renderComments(); renderPreviousComments();
    }, "button primary"), button(t("Close editor", "入力を閉じる"), () => { commentTarget = null; renderComments(); renderPreviousComments(); }, "button"));
    thread.append(el("p", "muted", t("Kept in this page. Export before closing.", "ページ内に保持します。閉じる前に書き出してください。")));
  } else thread.append(button(t("Add comment", "コメントを追加"), () => openComment(target, thread.closest("diffs-container") ?? thread.parentElement), "button"));
  return thread;
}
function annotations(path) {
  const targets = [...notes, ...(commentTarget ? [commentTarget] : []), ...(state.reference?.path === path ? [{ reference: state.reference }] : []),
    ...data.items.flatMap((selected) => selected.refs.filter((ref) => ref.path === path).map((ref) => ({ reference: ref })))];
  const unique = new Map(targets.filter((target) => target.reference?.path === path && (!target.snapshot || target.snapshot.id === data.id)).map((target) => [targetKey(target), target.reference]));
  return [...unique.values()].map((r) => ({ lineNumber: r.line, side: r.side === "old" ? "deletions" : "additions", metadata: {} }));
}
function renderComments(updateDiffs = true) {
  document.querySelectorAll(".comment-anchor:not([data-range])").forEach((anchor) => {
    if (anchor.reviewEnd != null) return;
    const thread = renderThread(anchor.reviewTarget);
    anchor.replaceChildren(...(thread ? [thread] : []));
  });
  for (const markdown of document.querySelectorAll(".markdown")) {
    const anchors = [...markdown.querySelectorAll(".comment-anchor")];
    for (const anchor of anchors) anchor.replaceChildren();
    const targets = new Map([...notes, ...(commentTarget ? [commentTarget] : [])]
      .filter((target) => target.reference).map((target) => [targetKey(target), target]));
    for (const target of targets.values()) {
      const ref = displayReference(target);
      if (!ref) continue;
      const candidates = anchors.filter((anchor) => {
        const r = anchor.reviewTarget.reference;
        return r.path === ref.path && r.side === ref.side && r.line <= ref.line && anchor.reviewEnd >= ref.line;
      });
      // A list item can contain a paragraph: use the smallest, innermost block.
      candidates.sort((a, b) => (a.reviewEnd - a.reviewTarget.reference.line) - (b.reviewEnd - b.reviewTarget.reference.line)
        || (a.reviewBlock.contains(b.reviewBlock) ? 1 : -1));
      const owner = candidates[0];
      if (owner) { const thread = renderThread(target); if (thread) owner.append(thread); }
    }
  }
  if (updateDiffs) for (const { instance, path } of renderers) instance.render({ lineAnnotations: annotations(path) });
}

function notesReport() { return {version:1,snapshot:comparisonIdentity(),notes}; }
function notesContent(format, report=notesReport()) {
  if (format === "json") return JSON.stringify(report,null,2);
  const snapshot=report.snapshot;
  return [
    `# ${data.title}`,`Snapshot: ${snapshot.id}`,`Base: ${snapshot.base}`,`Target: ${snapshot.target}`,`Captured: ${new Date(snapshot.captured_at*1000).toISOString()}`,
    ...report.notes.map((n,i)=>`## ${i+1}. ${n.reference ? `${n.reference.path}:${n.reference.line} (${n.reference.side})` : `Item: ${n.item}`}\n\nComment: ${n.id ?? "—"}\nStatus: ${n.resolution ? "resolved" : "open"}\nItem: ${n.item ?? "—"}\n\n${n.snapshot ? `Snapshot: ${n.snapshot.id}\nBase: ${n.snapshot.base}\nTarget: ${n.snapshot.target}\nCaptured: ${new Date(n.snapshot.captured_at*1000).toISOString()}\n\n` : ""}${n.reference?.quote ? `> ${n.reference.quote}\n\n` : ""}${n.text}${n.resolution ? `\n\nResolved in: ${n.resolution.snapshot.id}\nRevision: ${n.resolution.snapshot.revision}\n${n.resolution.summary}` : ""}`),
  ].join("\n\n");
}
function notesFeedback(message) {
  notesMessage=message;
  const status=document.querySelector(".notes-feedback");if(status)status.textContent=message;
}
async function copyNotes() {
  const saved=JSON.stringify(notes);
  try {
    if (!navigator.clipboard?.writeText) throw new Error("Clipboard unavailable");
    await navigator.clipboard.writeText(notesContent("md"));
    if(saved===JSON.stringify(notes)) exported=true;
    notesFeedback(t("Notes copied", "メモをコピーしました"));
  } catch {
    notesFeedback(t("Could not copy. Use Export notes to save a file.", "コピーできませんでした。「メモを書き出す」でファイルに保存できます。"));
  }
}
async function sendNotes() {
  const report=notesReport();const content=JSON.stringify(report);const saved=JSON.stringify(notes);
  if(!notes.length){notesFeedback(t("Add a comment before sending", "コメントを追加してから渡してください"));return;}
  if(!feedbackAttempt || feedbackAttempt.content!==content) feedbackAttempt={id:crypto.randomUUID(),content};
  notesFeedback(t("Sending to the server…", "サーバーへ渡しています…"));
  try {
    const response=await fetch("/api/feedback",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({id:feedbackAttempt.id,report})});
    const result=await response.json();if(!response.ok || result.delivered!==true || result.id!==feedbackAttempt.id)throw new Error(result.error || "Missing delivery receipt");
    if(saved===JSON.stringify(notes))exported=true;
    notesFeedback(t("Sent to the launch output. The agent can read it there.", "起動元の実行出力へ渡しました。Agentが出力を読むと受け取れます。"));
  }catch{
    notesFeedback(t("Could not send. Retry, or copy the notes.", "渡せませんでした。再試行するか、メモをコピーしてください。"));
  }
}
function exportNotes(format) {
  const content=notesContent(format);
  const url=URL.createObjectURL(new Blob([content],{type:format==="json"?"application/json":"text/markdown;charset=utf-8"}));
  const a=el("a");a.href=url;a.download=`sdev-${data.id}-notes.${format}`;a.click();
  setTimeout(()=>URL.revokeObjectURL(url),1000);exported=true;
}

function comparisonIdentity() {
  return { id:data.id, base:data.base, target:data.target, captured_at:data.captured_at, ...(data.revision ? {revision:data.revision} : {}) };
}
function displayReference(target) {
  const ref = target.reference;
  if (!target.snapshot || target.snapshot.id === data.id) return ref;
  if (!ref?.quote) return null;
  const card = [...document.querySelectorAll(".document-card")].find((card)=>card.dataset.path === ref.path);
  if (!card || state.sides[fileMap.get(ref.path)?.role] !== ref.side) return null;
  const matches = [...card.querySelectorAll(".markdown [data-line]")].filter((block)=>block.dataset.quote === ref.quote);
  return matches.length === 1 ? {...ref,line:Number(matches[0].dataset.line)} : null;
}
function renderPreviousComments() {
  document.querySelector(".previous-comments")?.remove();
  const targets = [...new Map([...notes, ...(commentTarget ? [commentTarget] : [])]
    .filter((target)=>target.snapshot && target.snapshot.id !== data.id && !displayReference(target))
    .map((target)=>[targetKey(target), target])).values()];
  if (!targets.length) return;
  const section = el("section", "previous-comments");
  section.append(el("h2",null,t("Comments on earlier text", "更新前の文章へのコメント")),
    el("p","muted",t("The original target and comparison are retained. These comments are not attached to changed text.", "元の文章と比較対象を保持しています。変更後の文章への指摘としては扱いません。")));
  for (const target of targets) { const thread=renderThread(target); if(thread) section.append(thread); }
  document.querySelector(".reading-area")?.append(section);
}
function updateStatus(message) {
  updateMessage = message;
  const status = document.querySelector(".update-status");
  if (status) { status.textContent = message; status.setAttribute("role","status"); }
}
function readingLocation() {
  const focused = document.activeElement?.closest?.(".markdown [data-line]");
  const active = document.querySelector(".pane.current")?.contains(focused) ? focused : null;
  const blocks = [...document.querySelectorAll(".pane.current .markdown [data-line]")].filter((block)=>!block.querySelector("[data-line]"));
  const top = state.overview ? (document.querySelector(".pane.current .pane-scroll")?.getBoundingClientRect().top ?? 0) : 90;
  const block = active ?? blocks.find((block)=>block.getBoundingClientRect().bottom > top);
  if (!block) return null;
  return {path:block.closest(".document-card").dataset.path, line:Number(block.dataset.line),
    quote:block.dataset.quote, top:block.getBoundingClientRect().top, focus:Boolean(active)};
}
function applyUpdate(next) {
  if (next.revision === data.revision) return;
  if (document.activeElement?.matches("textarea") || composing) {
    pendingUpdate = next;
    updateStatus(t("Changes ready; finish editing to apply", "更新あり・入力を終えると反映"));
    return;
  }
  pendingUpdate = null;
  const location = readingLocation();
  const hadDetails = Boolean(preview);
  const restartScan = scan.running || (scan.total !== null && scan.scanned >= scan.total);
  rememberPagePosition();
  Object.assign(data, next);
  document.title = `${data.title} · sdev review`;
  fileMap.clear(); for (const file of data.files) fileMap.set(file.path,file);
  for (const role of roles) {
    if (!currentFile(role)) state.paths[role] = preferredFiles(role)[0]?.path;
    if (currentFile(role)?.[state.sides[role]] == null) state.sides[role] = currentFile(role)?.new != null ? "new" : "old";
  }
  // Old judgments and scan cursors belong to the old input, including pending responses.
  relatedCache.clear(); history=[]; future=[];
  scan = {stopping:false,recent:[],withoutCandidate:0,running:false,scanned:0,total:null,cursor:0,candidates:[],error:null,partial:false,skipped:0};
  state.reference = null;
  if (state.item && !item()) state.item = null;
  updateMessage=t("Updated · judgments use the current text", "更新済み・現在の本文で判定");
  render();
  if (restartScan && (data.code_scope?.total ?? 0) > 0) runScan();
  requestAnimationFrame(()=>{
    if (!location) return;
    const card = [...document.querySelectorAll(".document-card")].find((card)=>card.dataset.path === location.path);
    const blocks = [...card?.querySelectorAll(".markdown [data-line]") ?? []];
    const matches=location.quote ? blocks.filter((block)=>block.dataset.quote === location.quote) : [];
    const exact=matches.length === 1 ? matches[0] : null;
    const block=exact ?? sourceOwner(blocks,location.line);
    if (!block) return;
    const offset = block.getBoundingClientRect().top-location.top;
    if (state.overview) block.closest(".pane-scroll").scrollTop += offset;
    else window.scrollBy({top:offset,behavior:"instant"});
    if (location.focus) block.focus({preventScroll:true});
    if (hadDetails && exact) showPreview(block.reviewEntries ?? [],block,true,block.reviewSection);
  });
}
async function refreshReview() {
  try {
    const response = await fetch("/api/refresh", {method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({revision:data.revision})});
    const result = await response.json();
    if (!response.ok) throw new Error(result.error || "Refresh failed");
    if (result.data) applyUpdate(result.data);
    let changed = false;
    for (const resolution of result.resolutions ?? []) {
      const note = notes.find((note) => note.id === resolution.id);
      if (!note || !resolution.snapshot?.id || typeof resolution.summary !== "string") continue;
      const next = {snapshot:resolution.snapshot,summary:resolution.summary};
      if (JSON.stringify(note.resolution) === JSON.stringify(next)) continue;
      note.resolution = next; changed = true;
    }
    if (changed) { exported = false; pendingResolutionRender = true; }
    if (pendingResolutionRender && !document.activeElement?.matches("textarea") && !composing) {
      pendingResolutionRender = false; renderComments(); renderPreviousComments();
    }
    if (result.error) updateStatus(`${t("Update failed; showing earlier text", "更新失敗・前の本文を表示中")}: ${result.error}`);
    else if (pendingUpdate) updateStatus(t("Changes ready; finish editing to apply", "更新あり・入力を終えると反映"));
    else if (data.hot) updateStatus(t("Auto update connected", "自動更新に接続"));
  } catch {
    updateStatus(t("Update disconnected; retrying automatically", "自動更新が切断・再接続中"));
  } finally {
    if (data.hot || data.server) setTimeout(refreshReview, 1000);
  }
}
function render() {
  closePreview();
  for (const { instance } of renderers) instance.cleanUp(); renderers = [];
  app.replaceChildren();
  const main = el("main", "workspace"); main.append(renderHeader());
  const summary = el("div", "comparison-summary");
  summary.append(el("span", "snapshot-dot"), el("strong", null, data.title), el("span", "comparison-ref", `${data.base.slice(0, 8)} → ${data.target === "working-tree" ? t("Working tree", "作業ツリー") : data.target.slice(0, 8)}`), badge(data.hot ? t("Auto update", "自動更新") : t("Snapshot", "取り込み時の状態")), el("span", "capture-time", new Date(data.captured_at * 1000).toLocaleString(data.language)));
  summary.append(el("span", "update-status", updateMessage));
  summary.title = `${data.id}\n${data.base}\n${data.target}`; main.append(summary);
  for (const warning of data.warnings) main.append(el("div", "warning", warning));
  const body = el("div", "review-body");
  const reading = el("div", "reading-area");
  const unanchored = data.items.filter((selected) => !selected.refs.length);
  if (unanchored.length) {
    const questions = el("section", "unanchored-questions");
    questions.append(el("h2", null, t("Questions without a specification passage", "対応する仕様がまだない問い")));
    for (const selected of unanchored) {
      const entry = el("section", "unanchored-item"); entry.dataset.unanchoredItem = selected.id; entry.tabIndex = 0;
      entry.append(el("h3", null, selected.title), renderMaterial(selected)); questions.append(entry);
    }
    reading.append(questions);
  }
  const stack = el("div", `stack ${state.overview ? "overview" : ""}`);
  if (!data.files.length) stack.append(el("div", "empty-state", data.comparison_has_changes
    ? t("Changes were captured, but no specifications were included. Add context with --spec PATH and ask the agent for specification questions.", "変更は取り込まれていますが、仕様文書がありません。--spec PATHで材料を追加し、Agentに仕様への問いを調べてもらってください。")
    : t("No changes in this comparison.", "この比較に変更はありません。")));
  else for (const role of roles) stack.append(renderPane(role));
  if (state.scan) { const panel = el("section", "scan-panel"); panel.id = "scan-panel"; reading.append(panel); }
  else reading.append(stack);
  body.append(reading); main.append(body); app.append(renderSidebar(), main);
  if (state.scan) renderScan();
  renderComments();
  renderPreviousComments();
  requestAnimationFrame(() => {window.scrollTo({ top: state.pageScroll ?? 0, left: 0, behavior: "instant" });updateOutlinePosition();});
}
function api(path, payload) {
  const revision = data.revision;
  const request = serverQueue.catch(() => {}).then(async () => {
    if (revision !== data.revision) throw new Error(t("Comparison updated", "比較対象が更新されました"));
    const response = await fetch(path, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({...payload,...(revision ? {revision} : {})}) });
    const result = await response.json();
    if (revision !== data.revision || (result.revision && result.revision !== revision)) throw new Error(t("Comparison updated", "比較対象が更新されました"));
    if (!response.ok) throw new Error(result.error || t("Judgment failed", "判定に失敗しました"));
    return result;
  });
  serverQueue = request; return request;
}
function relatedPassage(match) {
  if (match.table && Array.isArray(match.table.headers) && Array.isArray(match.table.rows)) {
    const wrapper = el("div", "related-table-container");
    const table = el("table", "related-table");
    const head = el("thead"); const headerRow = el("tr");
    for (const text of match.table.headers) { const cell = el("th", null, text); cell.scope = "col"; headerRow.append(cell); }
    head.append(headerRow); table.append(head);
    const body = el("tbody");
    const highlighted = new Set((match.table.highlighted_rows ?? []).filter((index) => Number.isInteger(index) && index >= 0 && index < match.table.rows.length));
    for (const [index, cells] of match.table.rows.entries()) {
      const row = el("tr", highlighted.has(index) ? "related-row-highlight" : null);
      for (const text of cells) row.append(el("td", null, text));
      body.append(row);
    }
    table.append(body); wrapper.append(table);
    if (match.truncated) wrapper.append(el("small", "muted", t("Only captured rows are shown.", "取り込めた行だけを表示しています。")));
    return wrapper;
  }
  const node = el("blockquote", "related-passage");
  const text = match.text;
  let position = 0;
  for (const range of match.highlights ?? []) {
    if (!Number.isInteger(range.start) || !Number.isInteger(range.end) || range.start < position || range.end <= range.start || range.end > text.length || text.slice(range.start, range.end) !== range.text) continue;
    node.append(document.createTextNode(text.slice(position, range.start)), el("mark", "related-highlight", text.slice(range.start, range.end)));
    position = range.end;
  }
  node.append(document.createTextNode(text.slice(position)));
  return node;
}
async function loadRelated(reference, target) {
  if (!target.isConnected) return;
  const key = JSON.stringify([data.revision, reference]);
  if (!relatedCache.has(key)) relatedCache.set(key, api("/api/related", reference).catch((error) => { relatedCache.delete(key); throw error; }));
  try {
    const result = await relatedCache.get(key); if (!target.isConnected) return;
    target.replaceChildren();
    let more = null;
    const matches = result.matches.filter((match) => typeof match.probability === "number" && match.probability >= .8);
    for (const [index, match] of matches.entries()) {
      const card = el("section", "related-result");
      const ref = { path: match.path, line: match.line, side: match.side };
      if (match.quote) ref.quote = match.quote;
      card.append(button(`${match.heading || match.path} ↗`, () => { state.scan = false; pickReference(ref); }, "trace-endpoint"),
        el("small", "trace-location", `${match.path}:${match.line}`), relatedPassage(match));
      if (index < 3) target.append(card);
      else {
        if (!more) {
          more = el("details", "more-related");
          more.append(el("summary", null, t(`Show ${matches.length - 3} more passages`, `ほかの関連箇所 ${matches.length - 3}件`)));
          target.append(more);
        }
        more.append(card);
      }
    }
    if (!matches.length) target.append(el("p", "muted", t("No clear requirement-to-design correspondence was found in the inspected candidates.", "調べた候補から、明確に対応する箇所は見つかりませんでした。")));
    if (result.highlight_error) {
      target.append(el("p", "muted", t("Emphasis is unavailable. The captured passages are still shown.", "対応箇所の強調を取得できませんでした。本文を表示しています。")), button(t("Retry emphasis", "強調を再取得"), () => { relatedCache.delete(key); loadRelated(reference, target); }, "button"));
    }
    target.append(el("p", "muted", `${result.tested}/${result.total} ${t("passages checked", "箇所を照合")} · ${result.model} · ${result.cache.misses === 0 ? t("Cached", "キャッシュ") : "Jev"}${result.partial ? t(" · Partial input", " · 一部の候補のみ") : ""}`));
  } catch (error) {
    if (!target.isConnected) return;
    target.replaceChildren(el("p", "warning", error.message), button(t("Retry", "再試行"), () => loadRelated(reference, target), "button"));
  }
}
function judgmentLabel(value, positive, negative) {
  return typeof value !== "number" ? t("Not available", "判定未取得") : value >= .7 ? positive : value <= .3 ? negative : t("Uncertain", "判断保留");
}
function scanScope() {
  return data.code_scope ?? { files:[], total:scan.total, specifications:data.files.filter((file)=>file.path.endsWith(".ja.md") === ja || !data.files.some((other)=>other.path === (file.path.endsWith(".ja.md") ? file.path.replace(".ja.md",".md") : file.path.replace(/\.md$/, ".ja.md")))), partial:false };
}
function renderScan() {
  const panel = document.getElementById("scan-panel"); if (!panel) return;
  const scope = scanScope(); const total = scan.total ?? scope.total;
  const status = document.querySelector(".scan-status");
  if (status) status.textContent = scan.total === null ? t("Not checked", "未検査") : `${scan.candidates.length} ${t("candidates", "件の候補")} · ${scan.scanned}/${total}`;
  panel.replaceChildren(el("h1", null, t("Does the code introduce behavior missing from the specification?", "コードに、仕様にない振る舞いが増えていないか")),
    el("p", "scan-intro", t("Check the captured changes against Requirements and Design. Inspect the returned code regions and decide whether a specification or implementation needs to change.", "取り込んだコード・設定の変更をRequirements／Designと照合します。確認候補の差分を読み、仕様や実装を直す必要があるか判断します。")));
  const inputs = el("div", "scan-inputs");
  const changes = el("section", "scan-input");
  changes.append(el("h2", null, t("Changes to inspect", "調べる変更")), el("p", null, `${scope.files.length} ${t("files", "ファイル")} · ${scope.total ?? "?"} ${t("change regions", "変更箇所")}`));
  const files = el("details"); files.append(el("summary",null,t("Show changed files", "変更ファイルを見る")));
  for (const file of scope.files) files.append(el("p", "scan-file", `${file.path} · ${file.notice ? t("Unavailable", "本文未取得") : `${file.regions} ${t("regions", "箇所")}`}`));
  changes.append(files);
  const specifications = el("section", "scan-input"); specifications.append(el("h2", null, t("Specifications to compare", "照合する仕様")));
  for (const file of scope.specifications) specifications.append(button(`${file.role === "requirements" ? "Requirements" : "Design"} · ${file.path}`, () => go(()=>{ state.scan=false; state.active=file.role; state.paths[file.role]=file.path; state.sides[file.role]="new"; state.reference=null; }), "scan-spec-link"));
  if (!scope.specifications.length) specifications.append(el("p", "warning", t("No specification captured", "仕様が取り込まれていません")));
  inputs.append(changes,specifications); panel.append(inputs);
  const questions = el("ol", "scan-questions");
  questions.append(el("li",null,t("Does this change affect behavior or an important design choice that belongs in a specification? Helpers and refactoring alone do not count.", "仕様に残す振る舞い・重要な設計判断を変えているか？ 補助関数や整理だけなら候補にしません。")),el("li",null,t("Do the supplied Requirements or Design describe that change? Unclear answers remain uncertain.", "今のRequirements／Designに、その内容が書かれているか？ 判断できないものは保留にします。")));
  panel.append(questions);
  const control = button(scan.stopping ? t("Stopping after this batch…", "この照合が終わったら停止…") : scan.running ? t("Stop after this batch", "この照合で停止") : scan.total !== null && scan.scanned >= scan.total ? t("Check again (use cache)", "再確認（キャッシュを利用）") : scan.scanned ? t("Continue checking", "続きから調べる") : t("Check changed code", "変更したコードを調べる"), () => {
    if (scan.running) { scan.stopping=true; renderScan(); }
    else { if (scan.total !== null && scan.scanned >= scan.total) scan = { stopping:false,recent:[],withoutCandidate:0,running:false,scanned:0,total:null,cursor:0,candidates:[],error:null,partial:false,skipped:0 }; runScan(); }
  }, "button primary"); control.disabled = scan.stopping || scope.total === 0; panel.append(control);
  const progress = el("progress", "scan-meter"); progress.max=Math.max(total ?? 1,1); progress.value=scan.scanned; panel.append(progress);
  const phase = scan.running ? t("Checking", "照合中") : scan.error ? t("Failed; incomplete", "失敗・未完了") : scan.total !== null && scan.scanned >= scan.total ? t("Complete", "照合完了") : scan.scanned ? t("Stopped; incomplete", "停止・未完了") : t("Not checked", "未検査");
  panel.append(el("p", "scan-progress", `${phase} · ${scan.scanned}/${total ?? "?"} ${t("regions checked", "箇所を照合")} · ${scan.candidates.length} ${t("candidates", "件の確認候補")} · ${scan.withoutCandidate} ${t("without a candidate", "件は追加確認の候補なし")}`));
  if (scan.running) {
    let offset=0; const active=[];
    for (const file of scope.files) { const end=offset+file.regions; if (end>scan.cursor && offset<scan.cursor+4) active.push(file.path); offset=end; }
    if (active.length) panel.append(el("p","scan-current",`${t("Now comparing", "いま照合している変更")}: ${active.join(" / ")}`));
  }
  if (scan.partial || scope.partial) panel.append(el("p","warning",t("Only part of the specification fit in this comparison. Missing coverage cannot be established from this input.","仕様の一部だけを照合しています。この範囲で記載が見つからなくても、仕様全体にないとは確定できません。")));
  const skipped=scan.skipped || scope.skipped_files || 0;
  if (skipped) panel.append(el("p","warning",`${skipped} ${t("files have unavailable text and were not checked", "ファイルは本文を取得できず、照合していません")}`));
  if (scan.error) panel.append(el("p", "warning", scan.error));
  if (scan.recent.length) {
    const recent=el("details","scan-recent");recent.append(el("summary",null,t("What the last comparison checked", "直近に照合した変更と結果")));
    for(const result of scan.recent) recent.append(el("p",null,`${result.path}:${result.line} · ${t("Needs specification", "仕様に残す変更")}: ${judgmentLabel(result.needs_spec,t("Yes","はい"),t("No","いいえ"))} · ${t("Documented", "記載")}: ${judgmentLabel(result.covered,t("Found","あり"),t("Not found in inspected scope","照合範囲では見つからない"))}`));
    panel.append(recent);
  }
  if (scan.candidates.length) panel.append(el("h2", "scan-results-heading",t("Changes to review", "確認する変更")));
  for (const candidate of scan.candidates) {
    const card = el("article", "code-candidate");
    card.append(badge(candidate.kind === "uncovered" ? t("No specification found · candidate", "記載を確認できない・候補") : t("Uncertain; inspect this change", "判断保留・この変更を確認"), "amber"), el("h3", null, `${candidate.path}:${candidate.line} (${candidate.side})`));
    const judgments=el("dl","candidate-judgments");
    judgments.append(el("dt",null,t("A change that belongs in a specification?", "仕様に残す変更か")),el("dd",null,judgmentLabel(candidate.needs_spec,t("Yes", "はい"),t("No", "いいえ"))),el("dt",null,t("Is it described in the supplied specifications?", "照合した仕様に記載があるか")),el("dd",null,judgmentLabel(candidate.covered,t("Found", "あり"),t("Not found in inspected scope", "照合範囲では見つからない"))));
    card.append(judgments,el("p","candidate-question",candidate.needs_spec < .7 ? t("Does this change introduce behavior or a design choice that should be recorded?", "この変更は、仕様に残す振る舞いや設計判断でしょうか？") : t("Confirm whether an existing requirement or design covers this change, or whether one needs to be added.", "既存の要求・設計がこの変更を含むか、追記が必要かを確認してください。")));
    const diffDetails=el("details","candidate-code"); diffDetails.open=scan.candidates.indexOf(candidate)===0;
    diffDetails.append(el("summary",null,t("Read the actual code difference", "該当するコード差分を読む")));
    const diff = el("pre", "candidate-diff");
    for (const line of candidate.diff.split("\n")) diff.append(el("div", line.startsWith("+") ? "addition" : line.startsWith("-") ? "deletion" : "", line || " "));
    diffDetails.append(diff); card.append(diffDetails);
    const target = { reference: { path:candidate.path,line:candidate.line,side:candidate.side }, item:null };
    card.append(button(t("Comment", "コメントを書く"), () => openComment(target, card), "button"), commentAnchor(target)); panel.append(card);
  }
  if (scan.total !== null && !scan.running && !scan.candidates.length && !scan.error) panel.append(el("p", "muted", t("No candidate was returned in the inspected scope. This is not proof that the specifications cover every change.", "調べた範囲では候補は返りませんでした。すべての変更が仕様に対応するという証明ではありません。")));
  renderComments(false);
}
async function runScan() {
  const runningScan = scan;
  scan.running = true; scan.stopping=false; scan.error = null; renderScan();
  while (scan.running) {
    try {
      const result = await api("/api/uncovered", { cursor:scan.cursor });
      if (scan !== runningScan) return;
      scan.scanned = result.scanned; scan.total = result.total; scan.candidates.push(...result.candidates);
      scan.recent=result.outcomes ?? []; scan.withoutCandidate+=scan.recent.filter((entry)=>entry.kind === "covered").length;
      scan.partial ||= result.partial; scan.skipped = result.skipped_files;
      if (result.next_cursor == null) scan.running = false; else scan.cursor = result.next_cursor;
      if(scan.stopping) scan.running=false;
      scan.stopping=false; renderScan();
      if (scan.running) await new Promise((resolve) => setTimeout(resolve, 80));
    } catch (error) { if (scan !== runningScan) return; scan.error = error.message; scan.running = false; scan.stopping=false; renderScan(); }
  }
}
document.addEventListener("keydown", (event) => {
  if (event.defaultPrevented || event.metaKey || event.ctrlKey || event.altKey || !["j","k"].includes(event.key.toLowerCase()) || event.target.closest("input,textarea,select,[contenteditable=true]")) return;
  const readingPane = document.querySelector(".pane.current") ?? document.activeElement.closest?.(".pane");
  const blocks = [...readingPane?.querySelectorAll(".markdown [data-line]") ?? []].filter((block) => !block.querySelector("[data-line]"));
  if (!blocks.length) return;
  const active = document.activeElement.closest?.("[data-line]");
  const current = blocks.indexOf(active);
  const top = state.overview ? readingPane.querySelector(".pane-scroll").getBoundingClientRect().top : (document.querySelector(".topbar")?.getBoundingClientRect().bottom ?? 0);
  const index = current < 0 ? Math.max(0, blocks.findIndex((block) => block.getBoundingClientRect().bottom > top))
    : Math.max(0, Math.min(blocks.length - 1, current + (event.key.toLowerCase() === "j" ? 1 : -1)));
  event.preventDefault(); closePreview();
  const next = blocks[index]; next.focus({ preventScroll:true }); next.scrollIntoView({ block:"center", behavior:"instant" });
  showPreview(next.reviewEntries ?? [], next, true, next.reviewSection);
});
document.addEventListener("click", (event) => {
  if (preview && !preview.node.contains(event.target) && !preview.trigger.contains(event.target)) closePreview();
});
document.addEventListener("keydown", (event) => { if (event.key === "Escape" && preview) { event.preventDefault(); closePreview(true); } });
window.addEventListener("resize", positionPreview);
window.addEventListener("scroll", positionPreview, true);
window.addEventListener("scroll", scheduleOutlinePosition, true);
document.addEventListener("focusin",(event)=>{const pane=event.target.closest?.(".pane");if(pane)activateReadingPane(pane);});
window.addEventListener("beforeunload", (event) => { if (!exported || [...drafts.values()].some((draft) => draft.trim())) { event.preventDefault(); event.returnValue = ""; } });
render();

if (data.hot) {
  document.addEventListener("compositionstart",()=>{composing=true;});
  document.addEventListener("compositionend",()=>{composing=false;});
  document.addEventListener("focusout",()=>{setTimeout(()=>{if(pendingUpdate) applyUpdate(pendingUpdate);},0);});
}
if (data.hot || data.server) setTimeout(refreshReview,1000);
