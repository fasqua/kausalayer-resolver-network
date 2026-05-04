import axios from "axios";
import { createHash } from "crypto";
import fs from "fs";
import path from "path";

interface SourceEntry {
  domain: string;
  path: string;
  jsonPath: string;
}

/**
 * Load source registry from sources.json.
 * Re-reads file on every call so new sources are picked up without restart.
 */
function loadSourceRegistry(): SourceEntry[] {
  const sourcesPath = path.resolve(__dirname, "../sources.json");
  try {
    const raw = fs.readFileSync(sourcesPath, "utf8");
    return JSON.parse(raw) as SourceEntry[];
  } catch (err: any) {
    console.error("[fetcher] Failed to load sources.json:", err.message);
    return [];
  }
}

function sha256(data: string): string {
  return createHash("sha256").update(data).digest("hex");
}

/**
 * Find the matching source entry for a given domain hash.
 * Re-reads sources.json each time so new sources are detected without restart.
 */
export function resolveSource(domainHash: number[]): SourceEntry | null {
  const targetHex = Buffer.from(domainHash).toString("hex");
  const registry = loadSourceRegistry();

  for (const entry of registry) {
    const entryHash = sha256(entry.domain);
    if (entryHash === targetHex) {
      return entry;
    }
  }

  return null;
}

/**
 * Fetch data from a source API.
 * Returns the raw HTTP response body as string.
 */
export async function fetchSourceData(source: SourceEntry): Promise<string> {
  const url = "https://" + source.domain + source.path;
  console.log("[fetcher] Fetching:", url);

  try {
    const response = await axios.get(url, {
      headers: {
        "User-Agent": "KausaLayer-KRN/1.0",
        "Accept": "*/*",
      },
      timeout: 15000,
    });

    const body = typeof response.data === "string"
      ? response.data
      : JSON.stringify(response.data);

    console.log("[fetcher] Response size:", body.length, "bytes");
    return body;
  } catch (err: any) {
    console.error("[fetcher] Fetch failed:", err.message);
    throw err;
  }
}

/**
 * Add a new source to sources.json at runtime.
 */
export function registerSource(domain: string, apiPath: string, jsonPath: string): void {
  const sourcesPath = path.resolve(__dirname, "../sources.json");
  const registry = loadSourceRegistry();
  registry.push({ domain, path: apiPath, jsonPath });
  fs.writeFileSync(sourcesPath, JSON.stringify(registry, null, 2));
  console.log("[fetcher] Registered source:", domain + apiPath);
}
