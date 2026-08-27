import { convertFileSrc } from "@tauri-apps/api/core";

export function resolveCoverArt(urlOrPath: string | null | undefined): string | null {
    if (!urlOrPath) return null;
    const trimmed = urlOrPath.trim();
    if (!trimmed) return null;
    if (trimmed.startsWith("http://") || trimmed.startsWith("https://") || trimmed.startsWith("data:") || trimmed.startsWith("blob:")) {
        return trimmed;
    }
    return convertFileSrc(trimmed);
}
