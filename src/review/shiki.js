// A local plain-text highlighter keeps the portable review small and offline.
// Pierre still renders line, word, and addition/deletion differences.
export * from "shiki/core";
export { createJavaScriptRegexEngine } from "shiki/engine/javascript";
export { createOnigurumaEngine } from "shiki/engine/oniguruma";
export { createHighlighterCore as createHighlighter } from "shiki/core";
export const bundledLanguages = {};
