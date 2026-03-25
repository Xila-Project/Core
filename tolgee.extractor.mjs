/**
 * @typedef {import('@tolgee/cli/extractor').ExtractionResult} ExtractionResult
 */

import path from "node:path";

function normalizePath(fileName) {
  return fileName.replace(/\\/g, "/");
}

function resolveNamespace(fileName) {
  const relativePath = normalizePath(path.relative(process.cwd(), fileName));
  const sourceMarker = "/src/";
  const markerIndex = relativePath.lastIndexOf(sourceMarker);

  if (markerIndex <= 0) {
    return undefined;
  }

  return relativePath.slice(0, markerIndex);
}

/**
 * Extracts keys from Rust `translate!("<key>", ...)` and `translation!("<key>", ...)` macro calls.
 *
 * @param {string} code
 * @param {string} fileName
 * @returns {ExtractionResult}
 */
export default function extractRustTranslations(code, fileName) {
  if (!fileName.endsWith(".rs")) {
    return { keys: [] };
  }

  const namespace = resolveNamespace(fileName);
  const foundKeys = [];
  const translationCall =
    /\b(?:translate|translation)!\s*\(\s*(?:c)?"((?:\\.|[^"\\])+)"/g;

  let match;
  while ((match = translationCall.exec(code)) !== null) {
    const keyName = match[1];
    const line = code.slice(0, match.index).split("\n").length;

    foundKeys.push({
      keyName,
      namespace,
      line,
      file: fileName,
    });
  }

  return {
    keys: foundKeys,
  };
}
