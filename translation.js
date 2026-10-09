/**
 * --export     # write ./translations/<locale>.json (key -> English) to translate
 * --merge      # merge ./translations/<locale>.json back into the locales
 * --merge --overwrite       # also replace keys that already have a translation
 */

const fs = require("fs");
const path = require("path");

const args = process.argv.slice(2);

const EXPORT = args.includes("--export");
const MERGE = args.includes("--merge");
const OVERWRITE = args.includes("--overwrite");

const LOCALES_DIR = "./locales";
const TODO_DIR = "./translations";
const BASE_FILE = "en.json";

const isObject = (v) => v !== null && typeof v === "object" && !Array.isArray(v);
const hasOwn = (obj, key) => Object.prototype.hasOwnProperty.call(obj, key);

// Gets a nested value using a dot-notation key (e.g. "menu.file.open")
const getValueByPath = (obj, pathStr) =>
    pathStr.split(".").reduce((acc, part) => (acc == null ? undefined : acc[part]), obj);

// Sets a nested value using a dot-notation key (the structure must already exist)
function setByPath(obj, pathStr, value) {
    const parts = pathStr.split(".");
    const last = parts.pop();
    const parent = parts.reduce((acc, part) => (isObject(acc) ? acc[part] : undefined), obj);
    if (!isObject(parent) || !hasOwn(parent, last)) return false;
    parent[last] = value;
    return true;
}

// Turns a nested object into a flat { "a.b.c": value } object
function flatten(obj, prefix = "", out = {}) {
    for (const [k, v] of Object.entries(obj)) {
        const key = prefix ? `${prefix}.${k}` : k;
        if (isObject(v)) flatten(v, key, out);
        else out[key] = v;
    }
    return out;
}

/**
 * Builds a new object with the exact structure of `base`,
 * filled with values from `target` where they exist.
 */
function syncNode(base, target, prefix, stats) {
    const out = {};
    const targetObj = isObject(target) ? target : {};

    for (const key of Object.keys(base)) {
        const fullKey = prefix ? `${prefix}.${key}` : key;
        const baseValue = base[key];
        const exists = hasOwn(targetObj, key);
        const current = targetObj[key];

        if (isObject(baseValue)) {
            // Base expects a nested object
            out[key] = syncNode(
                baseValue,
                exists && isObject(current) ? current : undefined,
                fullKey,
                stats
            );
        } else if (exists && !isObject(current)) {
            // Base expects a leaf (string/array/etc.) and we already have one
            out[key] = current;
            if (current === "") stats.empty.push(fullKey);
        } else {
            // Missing (or wrong type: object where a string should be)
            out[key] = "";
            stats.added.push(fullKey);
            stats.empty.push(fullKey);
        }
    }

    // Anything in the target that isn't in the base gets dropped
    for (const key of Object.keys(targetObj)) {
        if (!hasOwn(base, key)) {
            stats.removed.push(prefix ? `${prefix}.${key}` : key);
        }
    }

    return out;
}

function readJson(filePath) {
    try {
        return JSON.parse(fs.readFileSync(filePath, "utf8"));
    } catch (err) {
        console.error(`💥 Could not read/parse ${filePath}: ${err.message}`);
        return null;
    }
}

// ---- Main ------------------------------------------------------------------
const basePath = path.join(LOCALES_DIR, BASE_FILE);
const baseJson = readJson(basePath);

if (!baseJson) process.exit(1);

const files = fs
    .readdirSync(LOCALES_DIR)
    .filter((f) => f.endsWith(".json") && f !== BASE_FILE);

function exportTodo(file, emptyKeys) {
    if (emptyKeys.length === 0) {
        console.log(`✅ ${file}: nothing to export`);
        return;
    }

    const todo = {};
    emptyKeys.forEach((key) => (todo[key] = getValueByPath(baseJson, key)));

    fs.mkdirSync(TODO_DIR, { recursive: true });
    const outPath = path.join(TODO_DIR, file);
    fs.writeFileSync(outPath, JSON.stringify(todo, null, 2) + "\n", "utf8");
    console.log(`📝 ${file}: ${emptyKeys.length} keys -> ${outPath}`);
}

function mergeTranslations(file, synced, stats) {
    const todoPath = path.join(TODO_DIR, file);
    if (!fs.existsSync(todoPath)) return false;

    const raw = readJson(todoPath);
    if (!raw) return false;

    for (const [key, value] of Object.entries(flatten(raw))) {
        const baseValue = getValueByPath(baseJson, key);

        // Key doesn't exist in the base file (typo or removed)
        if (baseValue === undefined || isObject(baseValue)) {
            stats.unknown.push(key);
            continue;
        }

        // Still not translated, leave it empty
        if (value == null || (typeof value === "string" && value.trim() === "")) continue;

        // Already has a translation: only replace it with --overwrite
        if (getValueByPath(synced, key) !== "" && !OVERWRITE) {
            stats.skipped.push(key);
            continue;
        }

        setByPath(synced, key, value);
        stats.merged.push(key);
    }

    stats.empty = stats.empty.filter((k) => !stats.merged.includes(k));
    return true;
}

let totalAdded = 0;
let totalRemoved = 0;

for (const file of files) {
    const filePath = path.join(LOCALES_DIR, file);
    const json = readJson(filePath);
    if (!json) continue;

    const stats = { added: [], removed: [], empty: [], merged: [], skipped: [], unknown: [] };
    const synced = syncNode(baseJson, json, "", stats);

    if (EXPORT) {
        exportTodo(file, stats.empty);
        continue;
    }

    if (MERGE && !mergeTranslations(file, synced, stats)) {
        console.log(`⏭️  ${file}: no translation file in ${TODO_DIR}, skipped`);
        continue;
    }

    const newContent = JSON.stringify(synced, null, 2) + "\n";
    const oldContent = fs.readFileSync(filePath, "utf8");
    const changed = newContent !== oldContent;

    totalAdded += stats.added.length;
    totalRemoved += stats.removed.length;

    if (!changed) {
        console.log(`✅ ${file}: already in sync`);
    } else {
        console.log(
            `\n🛠️  ${file}: +${stats.added.length} added, -${stats.removed.length} removed` +
            (MERGE ? `, ${stats.merged.length} merged` : "")
        );
        stats.added.forEach((k) => console.log(`   + ${k}`));
        stats.removed.forEach((k) => console.log(`   - ${k}`));
        stats.merged.forEach((k) => console.log(`   ✏️  ${k}`));

        fs.writeFileSync(filePath, newContent, "utf8");
    }

    if (stats.skipped.length > 0) {
        console.log(
            `   ⚠️  ${stats.skipped.length} keys skipped (already translated, use --overwrite to replace)`
        );
    }
    if (stats.unknown.length > 0) {
        console.log(`   ⚠️  ${stats.unknown.length} keys ignored (not in ${BASE_FILE}):`);
        stats.unknown.forEach((k) => console.log(`      ? ${k}`));
    }

    if (stats.empty.length > 0) {
        console.log(`   ⚠️  ${stats.empty.length} keys empty (untranslated)`);
    }
}

if (!EXPORT) {
    console.log(`\nDone. ${totalAdded} keys added, ${totalRemoved} keys removed across ${files.length} files.`);
}