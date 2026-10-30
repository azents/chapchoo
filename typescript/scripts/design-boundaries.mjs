import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, extname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import postcss from "postcss";

const referenceName = "@chapchoo/design-reference";
const ignored = new Set([
  "node_modules",
  "dist",
  ".next",
  ".turbo",
  "storybook-static",
]);
const extensions = new Set([
  ".ts",
  ".tsx",
  ".mts",
  ".cts",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".css",
]);

function* sourceFiles(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory() && !ignored.has(entry.name)) {
      yield* sourceFiles(path);
    } else if (entry.isFile() && extensions.has(extname(path))) {
      yield path;
    }
  }
}

function contains(directory, file) {
  const path = relative(directory, file);
  return (
    path === "" ||
    (!path.startsWith(`..${sep}`) && path !== ".." && !path.startsWith(sep))
  );
}

function moduleSpecifiers(file) {
  if (extname(file) === ".css") {
    return stylesheetSpecifiers(file);
  }
  const source = ts.createSourceFile(
    file,
    readFileSync(file, "utf8"),
    ts.ScriptTarget.Latest,
    true,
  );
  const result = [];
  function visit(node) {
    if (
      (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) &&
      node.moduleSpecifier &&
      ts.isStringLiteralLike(node.moduleSpecifier)
    ) {
      result.push(node.moduleSpecifier.text);
    }
    if (
      ts.isCallExpression(node) &&
      (node.expression.kind === ts.SyntaxKind.ImportKeyword ||
        (ts.isIdentifier(node.expression) &&
          node.expression.text === "require")) &&
      node.arguments[0] &&
      ts.isStringLiteralLike(node.arguments[0])
    ) {
      result.push(node.arguments[0].text);
    }
    if (
      ts.isImportTypeNode(node) &&
      ts.isLiteralTypeNode(node.argument) &&
      ts.isStringLiteralLike(node.argument.literal)
    ) {
      result.push(node.argument.literal.text);
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  return result;
}

function stylesheetSpecifiers(file) {
  const source = postcss.parse(readFileSync(file, "utf8"), { from: file });
  const result = [];
  source.walkAtRules(/^import$/i, (rule) => {
    const match = rule.params.match(
      /^(?:url\(\s*(?:"([^"]+)"|'([^']+)'|([^)\s]+))\s*\)|"([^"]+)"|'([^']+)')/i,
    );
    const specifier = match?.slice(1).find((value) => value);
    if (specifier) {
      result.push(specifier);
    }
  });
  source.walkDecls(/^(composes|compose-with)$/, (declaration) => {
    const match = declaration.value.match(/\bfrom\s+(?:"([^"]+)"|'([^']+)')/);
    const specifier = match?.[1] ?? match?.[2];
    if (specifier) {
      result.push(specifier);
    }
  });
  return result;
}

/** Check reference ownership, not the entire repository's dependency policy. */
export function checkDesignBoundaries(root) {
  const workspaces = [];
  for (const group of ["apps", "packages"]) {
    const directory = join(root, group);
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      const manifestPath = join(path, "package.json");
      if (entry.isDirectory() && existsSync(manifestPath)) {
        const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
        workspaces.push({ group, path, manifest });
      }
    }
  }
  const errors = [];
  function check(owner, target, location) {
    if (
      target &&
      owner !== target &&
      (target.manifest.name === referenceName ||
        (owner.manifest.name === referenceName && target.group === "apps"))
    ) {
      errors.push(
        `${location}: ${owner.manifest.name} must not depend on ${target.manifest.name}`,
      );
    }
  }
  for (const owner of workspaces) {
    for (const key of [
      "dependencies",
      "devDependencies",
      "peerDependencies",
      "optionalDependencies",
    ]) {
      for (const [name, value] of Object.entries(owner.manifest[key] ?? {})) {
        const target = workspaces.find(
          (candidate) =>
            candidate.manifest.name === name ||
            (typeof value === "string" &&
              (value.startsWith(`workspace:${candidate.manifest.name}@`) ||
                ((value.startsWith("file:") ||
                  value.startsWith("link:") ||
                  value.startsWith("workspace:.")) &&
                  resolve(owner.path, value.slice(value.indexOf(":") + 1)) ===
                    candidate.path))),
        );
        check(
          owner,
          target,
          `${relative(root, owner.path)}/package.json (${key})`,
        );
      }
    }
    const configPath = join(owner.path, "tsconfig.json");
    let options = {
      moduleResolution: ts.ModuleResolutionKind.Bundler,
      allowJs: true,
    };
    if (existsSync(configPath)) {
      const config = ts.readConfigFile(configPath, ts.sys.readFile);
      if (config.error) {
        throw new Error(`Cannot read ${configPath}`);
      }
      options = ts.parseJsonConfigFileContent(
        config.config,
        ts.sys,
        owner.path,
      ).options;
    }
    for (const file of sourceFiles(owner.path)) {
      for (const specifier of moduleSpecifiers(file)) {
        const resolved = ts.resolveModuleName(specifier, file, options, ts.sys)
          .resolvedModule?.resolvedFileName;
        const target = workspaces.find(
          (candidate) =>
            specifier === candidate.manifest.name ||
            specifier.startsWith(`${candidate.manifest.name}/`) ||
            (resolved && contains(candidate.path, resolve(resolved))) ||
            (specifier.startsWith(".") &&
              contains(candidate.path, resolve(dirname(file), specifier))),
        );
        check(owner, target, `${relative(root, file)} (${specifier})`);
      }
    }
  }
  return errors;
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const errors = checkDesignBoundaries(root);
  if (errors.length) {
    console.error(errors.join("\n"));
    process.exitCode = 1;
  } else {
    console.log("Design reference dependency boundaries passed.");
  }
}
