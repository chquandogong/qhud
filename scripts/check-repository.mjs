#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const errors = [];

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
      if (!gitObjectExists(`${match[1]}:${target}`)) {
        errors.push(`${note} links to ${target}, which does not exist at ${match[1]}.`);
      }
    }
  }
}

const englishDocs = sortedMarkdownFiles("docs");
compareFileSets(englishDocs, sortedMarkdownFiles("docs/i18n/ko/docs"), "Korean docs");
compareFileSets(englishDocs, sortedMarkdownFiles("docs/i18n/zh-CN/docs"), "Simplified Chinese docs");

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
    `${englishDocs.length} mirrored docs files, and local Markdown links are consistent.`,
);
