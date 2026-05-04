/**
 * URL sanitization function: returns standardized unique result for same-origin URLs, returns / for cross-origin URLs
 * @param url The URL string to process (supports absolute URLs, relative URLs, same-origin paths without origin prefix)
 * @returns Standardized same-origin path or / for cross-origin
 * @spec Compliant with WHATWG URL Standard & browser same-origin policy triple rule (protocol/host/port exact match)
 */
export function checkSameOrigin(url: string): string {
  // Fallback for non-browser environments (e.g., Node.js/SSR), return / directly
  if (typeof window === "undefined" || !window.location) {
    return "/";
  }

  const currentOrigin = window.location.origin;
  // Use current page URL as base to correctly resolve relative paths/origin-less URLs, fully aligning with browser parsing behavior
  const baseHref = window.location.href;

  let parsedUrl: URL;
  try {
    // Native URL API strictly follows WHATWG standard, automatically handles:
    // 1. Relative path resolution (./ ../ expansion, root path completion)
    // 2. Protocol/hostname lowercase normalization
    // 3. Default port auto-removal (http:80 / https:443)
    // 4. Path encoding normalization
    // 5. Credential info auto-stripping, no impact on origin check
    parsedUrl = new URL(url, baseHref);
  } catch {
    // Invalid URL parsing failed, treat as cross-origin, return /
    return "/";
  }

  // Same-origin check: strictly match origin serialization result, fully aligning with browser same-origin policy
  // Only considered same-origin if protocol, host, port triple are exactly the same
  const isSameOrigin = parsedUrl.origin === currentOrigin;

  if (!isSameOrigin) {
    // Cross-origin URLs (including special protocols like data:/javascript:/blob: not from current origin) return / uniformly
    return "/";
  }

  // Same-origin URLs return standardized full path (pathname + search + hash)
  // Native URL API has completed path normalization, ensuring different representations of the same resource output exactly the same
  // Example: ./test, /path/../test, https://current.com/test all output /test
  return parsedUrl.pathname + parsedUrl.search + parsedUrl.hash;
}
