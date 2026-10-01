#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const errors = [];
const options = parseArguments(process.argv.slice(2));

function parseArguments(argumentsList) {
  const usage =
    "Usage: node scripts/check-repository.mjs [--base <git-ref>]\n\n" +
    "  --base <git-ref>  Also require that every English source document changed since\n" +
    "                    <git-ref> changes together with its Korean and Simplified Chinese\n" +
    "                    mirrors, unless a commit trailer acknowledges an English-only change:\n" +
    "                    Translation-Exempt: <path> <reason>";
  const parsed = { base: undefined };
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === "--help" || argument === "-h") {
      console.log(usage);
      process.exit(0);
    } else if (argument === "--base" && /^[^-]/.test(argumentsList[index + 1] ?? "")) {
      parsed.base = argumentsList[index + 1];
      index += 1;
    } else if (/^--base=[^-]/.test(argument)) {
      parsed.base = argument.slice("--base=".length);
    } else {
      console.error(`Unrecognized or incomplete argument: ${argument}\n\n${usage}`);
      process.exit(2);
    }
  }
  return parsed;
}

function repositoryPath(relativePath) {
  return path.join(repositoryRoot, ...relativePath.split("/"));
}

function read(relativePath) {
  return fs.readFileSync(repositoryPath(relativePath), "utf8");
}

function capture(regex, input, description) {
  const match = regex.exec(input);
  if (!match) {
    errors.push(`Could not read ${description}.`);
    return undefined;
  }
  return match[1];
}

function sortedMarkdownFiles(rootRelativePath) {
  const rootPath = repositoryPath(rootRelativePath);
  const files = [];

  function visit(directory, prefix = "") {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      if (entry.name === "i18n" && rootRelativePath === "docs" && prefix === "") {
        continue;
      }
      const relativePath = prefix ? `${prefix}/${entry.name}` : entry.name;
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(absolutePath, relativePath);
      } else if (entry.isFile() && entry.name.endsWith(".md")) {
        files.push(relativePath);
      }
    }
  }

  visit(rootPath);
  return files.sort();
}

function compareFileSets(reference, candidate, candidateName) {
  const missing = reference.filter((file) => !candidate.includes(file));
  const extra = candidate.filter((file) => !reference.includes(file));
  if (missing.length > 0) {
    errors.push(`${candidateName} is missing: ${missing.join(", ")}`);
  }
  if (extra.length > 0) {
    errors.push(`${candidateName} has files absent from the English docs: ${extra.join(", ")}`);
  }
}

function gitObjectExists(specification) {
  return spawnSync("git", ["cat-file", "-e", specification], {
    cwd: repositoryRoot,
    stdio: "ignore",
  }).status === 0;
}

function repositoryMarkdownFiles() {
  const files = [];
  const excludedDirectories = new Set([".git", "node_modules", "target"]);

  function visit(directory, prefix = "") {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      if (entry.isDirectory() && excludedDirectories.has(entry.name)) {
        continue;
      }
      const relativePath = prefix ? `${prefix}/${entry.name}` : entry.name;
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(absolutePath, relativePath);
      } else if (entry.isFile() && entry.name.endsWith(".md")) {
        files.push(relativePath);
      }
    }
  }

  visit(repositoryRoot);
  return files.sort();
}

const cargoManifest = read("src-tauri/Cargo.toml");
const cargoVersion = capture(
  /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"\s*$/m,
  cargoManifest,
  "the Cargo package version",
);
const tauriVersion = JSON.parse(read("src-tauri/tauri.conf.json")).version;
const lockPackage = read("Cargo.lock")
  .split(/^\[\[package\]\]\s*$/m)
  .find((block) => /^name\s*=\s*"qhud"\s*$/m.test(block));
const lockVersion = lockPackage
  ? capture(/^version\s*=\s*"([^"]+)"\s*$/m, lockPackage, "the qhud Cargo.lock version")
  : undefined;

if (cargoVersion && (cargoVersion !== tauriVersion || cargoVersion !== lockVersion)) {
  errors.push(
    `Version mismatch: Cargo.toml=${cargoVersion}, tauri.conf.json=${tauriVersion}, Cargo.lock=${lockVersion}.`,
  );
}

if (!fs.readFileSync(repositoryPath("README.md")).equals(fs.readFileSync(repositoryPath("README.ko.md")))) {
  errors.push("README.md and README.ko.md must remain byte-for-byte identical.");
}

if (cargoVersion) {
  const tag = `v${cargoVersion}`;
  const releaseNotes = [
    `docs/05-ops/releases/${tag}.md`,
    `docs/i18n/ko/docs/05-ops/releases/${tag}.md`,
    `docs/i18n/zh-CN/docs/05-ops/releases/${tag}.md`,
  ];
  for (const note of releaseNotes) {
    if (!fs.existsSync(repositoryPath(note))) {
      errors.push(`Current release notes are missing: ${note}`);
      continue;
    }

    const noteText = read(note);
    if (!noteText.includes(tag)) {
      errors.push(`Current release notes do not identify ${tag}: ${note}`);
    }
    const languageBlock = /<!-- qhud:languages -->([\s\S]*?)<!-- \/qhud:languages -->/.exec(noteText)?.[1];
    if (!languageBlock) {
      errors.push(`Current release notes have no qhud language switcher: ${note}`);
      continue;
    }
    const actualLanguageLinks = [...languageBlock.matchAll(/href="([^"]+)"/g)]
      .map((match) => match[1])
      .sort();
    const expectedLanguageLinks = releaseNotes
      .filter((candidate) => candidate !== note)
      .map((candidate) => `https://github.com/chquandogong/qhud/blob/${tag}/${candidate}`)
      .sort();
    if (JSON.stringify(actualLanguageLinks) !== JSON.stringify(expectedLanguageLinks)) {
      errors.push(
        `${note} must link to the other release-note languages with absolute ${tag}-pinned GitHub URLs.`,
      );
    }
  }
  if (!read("CHANGELOG.md").includes(`## [${cargoVersion}]`)) {
    errors.push(`CHANGELOG.md has no entry for ${cargoVersion}.`);
  }
  for (const readme of ["README.md", "README.en.md", "README.ko.md", "README.zh-CN.md"]) {
    if (!read(readme).includes(`/releases/tag/${tag}`)) {
      errors.push(`${readme} does not link to the current release ${tag}.`);
    }
  }

  const installationDocs = [
    "docs/GUIDE.md",
    "docs/05-ops/WINDOWS.md",
    "docs/i18n/ko/docs/GUIDE.md",
    "docs/i18n/ko/docs/05-ops/WINDOWS.md",
    "docs/i18n/zh-CN/docs/GUIDE.md",
    "docs/i18n/zh-CN/docs/05-ops/WINDOWS.md",
  ];
  for (const document of installationDocs) {
    const staleExamples = [...read(document).matchAll(/\bqhud-v(\d+\.\d+\.\d+)(?=-)/g)]
      .map((match) => match[1])
      .filter((version) => version !== cargoVersion);
    if (staleExamples.length > 0) {
      errors.push(
        `${document} contains stale package examples: ${[...new Set(staleExamples)].join(", ")}.`,
      );
    }
  }
}

const releaseDirectory = repositoryPath("docs/05-ops/releases");
const releaseTags = fs
  .readdirSync(releaseDirectory)
  .filter((name) => /^v\d+\.\d+\.\d+\.md$/.test(name))
  .map((name) => name.slice(0, -3))
  .sort();
for (const releaseTag of releaseTags) {
  const notes = [
    `docs/05-ops/releases/${releaseTag}.md`,
    `docs/i18n/ko/docs/05-ops/releases/${releaseTag}.md`,
    `docs/i18n/zh-CN/docs/05-ops/releases/${releaseTag}.md`,
  ];
  for (const note of notes) {
    const languageBlock = /<!-- qhud:languages -->([\s\S]*?)<!-- \/qhud:languages -->/.exec(
      read(note),
    )?.[1];
    if (!languageBlock) {
      errors.push(`Release notes have no qhud language switcher: ${note}`);
      continue;
    }
    const links = [...languageBlock.matchAll(/href="([^"]+)"/g)].map((match) => match[1]);
    const targets = notes.filter((candidate) => candidate !== note);
    if (links.length !== targets.length) {
      errors.push(`${note} must link exactly once to each other release-note language.`);
      continue;
    }
    for (const target of targets) {
      const matches = links
        .map((link) =>
          /^https:\/\/github\.com\/chquandogong\/qhud\/blob\/(v\d+\.\d+\.\d+)\/(.+)$/.exec(
            link,
          ),
        )
        .filter((match) => match?.[2] === target);
      if (matches.length !== 1) {
        errors.push(`${note} must use an absolute immutable GitHub URL for ${target}.`);
        continue;
      }
      const match = matches[0];
      // The current version's tag can only be created after its preparation
      // PR merges, so until it exists the working tree stands in for it.
      const untaggedCurrent =
        cargoVersion &&
        match[1] === `v${cargoVersion}` &&
        !gitObjectExists(`refs/tags/${match[1]}`) &&
        fs.existsSync(repositoryPath(target));
      if (!untaggedCurrent && !gitObjectExists(`${match[1]}:${target}`)) {
        errors.push(`${note} links to ${target}, which does not exist at ${match[1]}.`);
      }
    }
  }
}

const englishDocs = sortedMarkdownFiles("docs");
compareFileSets(englishDocs, sortedMarkdownFiles("docs/i18n/ko/docs"), "Korean docs");
compareFileSets(englishDocs, sortedMarkdownFiles("docs/i18n/zh-CN/docs"), "Simplified Chinese docs");

// Translation policy: CONTRIBUTING.md, "Documentation and translations".
// Every English source below needs complete Korean and Simplified Chinese
// mirrors. A root Markdown page outside these lists must be named a
// canonical-language record, so a new page cannot silently skip the policy.
const translationLanguages = ["ko", "zh-CN"];
const mirroredRootDocuments = [
  "CHANGELOG.md",
  "CODE_OF_CONDUCT.md",
  "CONTRIBUTING.md",
  "SECURITY.md",
  "SUPPORT.md",
];
// README.md is the Korean mirror; README.ko.md is its byte-identical copy.
const readmeFamily = { source: "README.en.md", mirrors: ["README.md", "README.zh-CN.md"] };
const canonicalLanguageRootDocuments = new Set(["AGENTS.md", "README.ko.md"]);
const translationSets = [
  readmeFamily,
  ...mirroredRootDocuments.map((document) => ({
    source: document,
    mirrors: translationLanguages.map((language) => `docs/i18n/${language}/${document}`),
  })),
  ...englishDocs.map((document) => ({
    source: `docs/${document}`,
    mirrors: translationLanguages.map((language) => `docs/i18n/${language}/docs/${document}`),
  })),
];

{
  const classified = new Set([
    ...mirroredRootDocuments,
    readmeFamily.source,
    ...readmeFamily.mirrors,
    ...canonicalLanguageRootDocuments,
  ]);
  for (const entry of fs.readdirSync(repositoryRoot, { withFileTypes: true })) {
    if (entry.isFile() && entry.name.endsWith(".md") && !classified.has(entry.name)) {
      errors.push(
        `${entry.name} is neither a mirrored document nor a canonical-language record; ` +
          "classify it in scripts/check-repository.mjs and CONTRIBUTING.md.",
      );
    }
  }
  for (const language of translationLanguages) {
    const mirrorRoot = `docs/i18n/${language}`;
    const present = fs
      .readdirSync(repositoryPath(mirrorRoot), { withFileTypes: true })
      .filter((entry) => entry.isFile() && entry.name.endsWith(".md"))
      .map((entry) => entry.name)
      .sort();
    compareFileSets(mirroredRootDocuments, present, `${mirrorRoot}/`);
  }
}

// Facts that must survive translation unchanged. Prose may differ; these are
// compared mechanically between every English source and each mirror.
const languageSwitcherPattern = /<!-- qhud:languages -->[\s\S]*?<!-- \/qhud:languages -->/g;
// Translated headings carry their English anchor in an <a id> element.
const anchorPattern = /<!-- qhud:anchor -->\s*<a id="[^"]*"><\/a>/g;
const fencedBlockPattern = /^([ \t]*)(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1\2[ \t]*$/gm;

function comparableFacts(text) {
  const body = text.replace(languageSwitcherPattern, "").replace(anchorPattern, "");
  const prose = body.replace(fencedBlockPattern, "");
  const all = (pattern, input) => [...input.matchAll(pattern)].map((match) => match[0]).sort();
  return [
    [
      "requirement, decision, risk and assumption IDs",
      all(/\b(?:FR|NFR|D|CV)-\d+\b|\b[RA]\d{1,2}\b/g, prose),
    ],
    ["dates", all(/\b\d{4}-\d{2}-\d{2}\b/g, body)],
    ["release versions", [...new Set(all(/\bv?\d+\.\d+\.\d+\b/g, body))]],
    [
      "fenced code blocks",
      all(fencedBlockPattern, body).map((block) =>
        block
          .split("\n")
          .map((line) => line.trim())
          .join("\n"),
      ),
    ],
    ["table rows", [String((prose.match(/^[ \t]*\|.*\|[ \t]*$/gm) ?? []).length)]],
    ["headings", [String((prose.match(/^#{1,6}[ \t]/gm) ?? []).length)]],
  ];
}

function multisetDifference(left, right) {
  const remaining = new Map();
  for (const item of right) {
    remaining.set(item, (remaining.get(item) ?? 0) + 1);
  }
  const difference = [];
  for (const item of left) {
    if (remaining.get(item)) {
      remaining.set(item, remaining.get(item) - 1);
    } else {
      difference.push(item);
    }
  }
  return difference;
}

function describeFacts(items) {
  const shown = items.slice(0, 5).map((item) => JSON.stringify(item.split("\n")[0].slice(0, 60)));
  return shown.join(", ") + (items.length > shown.length ? `, and ${items.length - shown.length} more` : "");
}

let comparedMirrors = 0;
for (const { source, mirrors } of translationSets) {
  if (!fs.existsSync(repositoryPath(source))) {
    continue;
  }
  const sourceFacts = comparableFacts(read(source));
  for (const mirror of mirrors) {
    if (!fs.existsSync(repositoryPath(mirror))) {
      continue;
    }
    comparedMirrors += 1;
    const mirrorFacts = comparableFacts(read(mirror));
    sourceFacts.forEach(([kind, expected], index) => {
      const actual = mirrorFacts[index][1];
      if (kind === "table rows" || kind === "headings") {
        if (expected[0] !== actual[0]) {
          errors.push(`${mirror} has ${actual[0]} ${kind}; ${source} has ${expected[0]}.`);
        }
        return;
      }
      const missing = multisetDifference(expected, actual);
      const extra = multisetDifference(actual, expected);
      if (missing.length > 0 || extra.length > 0) {
        errors.push(
          `${mirror} ${kind} differ from ${source}` +
            (missing.length > 0 ? `; missing ${describeFacts(missing)}` : "") +
            (extra.length > 0 ? `; extra ${describeFacts(extra)}` : "") +
            ".",
        );
      }
    });
  }
}

// Pull-request signal: an English source changed since the base must change
// with both mirrors, or carry an explicit English-only acknowledgement.
let translationSummary = "";
if (options.base) {
  const git = (gitArguments) =>
    spawnSync("git", gitArguments, {
      cwd: repositoryRoot,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    });
  const diff = git(["diff", "--name-only", "--no-renames", "--diff-filter=d", "-z", `${options.base}...HEAD`]);
  const log = git(["log", "--format=%B%x00", `${options.base}..HEAD`]);
  if (diff.status !== 0 || log.status !== 0) {
    errors.push(
      `Could not compare HEAD with ${options.base}: ` +
        `${(diff.stderr || log.stderr || "").trim() || "git failed"}`,
    );
  } else {
    const changed = new Set(diff.stdout.split("\0").filter(Boolean));
    const exemptions = new Map();
    for (const match of log.stdout.matchAll(/^Translation-Exempt:[ \t]*(\S+)[ \t]+(\S.*)$/gm)) {
      exemptions.set(match[1], match[2].trim());
    }
    const annotate = process.env.GITHUB_ACTIONS === "true";
    let carried = 0;
    let acknowledged = 0;
    for (const { source, mirrors } of translationSets) {
      if (!changed.has(source)) {
        continue;
      }
      const unchanged = mirrors.filter((mirror) => !changed.has(mirror));
      if (unchanged.length === 0) {
        carried += 1;
      } else if (exemptions.has(source)) {
        acknowledged += 1;
        console.log(
          `${annotate ? `::notice file=${source}::` : "Note: "}${source} changed without ` +
            `${unchanged.join(" and ")}; acknowledged as English-only: ${exemptions.get(source)}`,
        );
      } else {
        const message =
          `${source} changed since ${options.base} without ${unchanged.join(" and ")}. ` +
          "Update the translation, or acknowledge an English-only change with the commit trailer " +
          `"Translation-Exempt: ${source} <reason>".`;
        errors.push(message);
        if (annotate) {
          console.log(`::error file=${source}::${message}`);
        }
      }
    }
    translationSummary =
      ` Since ${options.base}, ${carried} changed English source(s) carry both translations` +
      ` and ${acknowledged} are acknowledged as English-only.`;
  }
}

{
  const files = repositoryMarkdownFiles();
  const extractors = [
    /!?\[[^\]]*\]\(\s*(?:<([^>]+)>|([^\s)]+))(?:\s+["'][^)]*["'])?\s*\)/g,
    /\b(?:href|src)\s*=\s*["']([^"']+)["']/gi,
    /^\s*\[[^\]]+\]:\s*(?:<([^>]+)>|(\S+))/gm,
  ];

  for (const markdownFile of files) {
    const normalizedMarkdownFile = markdownFile.replaceAll("\\", "/");
    if (!fs.existsSync(repositoryPath(normalizedMarkdownFile))) {
      continue;
    }
    const content = read(normalizedMarkdownFile);
    for (const extractor of extractors) {
      extractor.lastIndex = 0;
      for (const match of content.matchAll(extractor)) {
        const rawTarget = match.slice(1).find(Boolean)?.trim();
        if (
          !rawTarget ||
          rawTarget.startsWith("#") ||
          rawTarget.startsWith("/") ||
          rawTarget.startsWith("//") ||
          /^[a-z][a-z0-9+.-]*:/i.test(rawTarget) ||
          rawTarget.includes("${{")
        ) {
          continue;
        }

        const targetWithoutFragment = rawTarget.split("#", 1)[0].split("?", 1)[0];
        if (!targetWithoutFragment) {
          continue;
        }

        let decodedTarget;
        try {
          decodedTarget = decodeURIComponent(targetWithoutFragment);
        } catch {
          errors.push(`${markdownFile} contains an invalid encoded link: ${rawTarget}`);
          continue;
        }
        const resolvedTarget = path.resolve(
          path.dirname(repositoryPath(normalizedMarkdownFile)),
          decodedTarget,
        );
        const line = content.slice(0, match.index).split("\n").length;
        const relativeToRepository = path.relative(repositoryRoot, resolvedTarget);
        if (
          relativeToRepository === ".." ||
          relativeToRepository.startsWith(`..${path.sep}`) ||
          path.isAbsolute(relativeToRepository)
        ) {
          errors.push(`${markdownFile}:${line} links outside the repository: ${rawTarget}`);
          continue;
        }
        if (!fs.existsSync(resolvedTarget)) {
          errors.push(`${markdownFile}:${line} links to a missing local path: ${rawTarget}`);
        }
      }
    }
  }
}

if (errors.length > 0) {
  console.error("Repository integrity check failed:\n");
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

console.log(
  `Repository integrity check passed for qhud ${cargoVersion}: versions, release metadata, install examples, ` +
    `${englishDocs.length} mirrored docs files, translation facts in ${comparedMirrors} mirrors, ` +
    `and local Markdown links are consistent.${translationSummary}`,
);
