// @ts-ignore
import { describe, it, expect } from "bun:test";
import { normalizeCanonicalString, getCanonicalKey, isCanonicalEntityMatch } from "../src/lib/utils/format";

describe("Canonical Normalization Utilities", () => {
    describe("normalizeCanonicalString", () => {
        it("handles null, undefined, and empty strings gracefully", () => {
            expect(normalizeCanonicalString(null)).toBe("");
            expect(normalizeCanonicalString(undefined)).toBe("");
            expect(normalizeCanonicalString("")).toBe("");
            expect(normalizeCanonicalString("   ")).toBe("");
        });

        it("collapses runs of whitespace and trims", () => {
            expect(normalizeCanonicalString("  Daft    Punk   ")).toBe("daft punk");
            expect(normalizeCanonicalString("Random\t\nAccess\n\nMemories")).toBe("random access memories");
        });

        it("normalizes unicode accents and diacritics via NFKD", () => {
            expect(normalizeCanonicalString("Beyoncé")).toBe("beyonce");
            expect(normalizeCanonicalString("Björk")).toBe("bjork");
            expect(normalizeCanonicalString("Sigur Rós")).toBe("sigur ros");
            expect(normalizeCanonicalString("Motörhead")).toBe("motorhead");
        });
    });

    describe("getCanonicalKey", () => {
        it("creates standardized artist::title keys", () => {
            expect(getCanonicalKey("Daft Punk", "Discovery")).toBe("daft punk::discovery");
            expect(getCanonicalKey("  Daft   Punk  ", "  Discovery  ")).toBe("daft punk::discovery");
        });

        it("handles missing artist or title", () => {
            expect(getCanonicalKey(null, "Discovery")).toBe("::discovery");
            expect(getCanonicalKey("Daft Punk", null)).toBe("daft punk::");
        });
    });

    describe("isCanonicalEntityMatch", () => {
        it("matches identical entities regardless of whitespace or casing", () => {
            const a = { title: "Random Access Memories", artist: "Daft Punk" };
            const b = { title: "  random   access memories ", artist: "daft punk" };
            expect(isCanonicalEntityMatch(a, b)).toBe(true);
        });

        it("matches entities with unicode accent differences", () => {
            const a = { title: "Renaissance", artist: "Beyoncé" };
            const b = { title: "Renaissance", artist: "Beyonce" };
            expect(isCanonicalEntityMatch(a, b)).toBe(true);
        });

        it("rejects different titles or different artists", () => {
            const a = { title: "Discovery", artist: "Daft Punk" };
            const b = { title: "Discovery", artist: "Justice" };
            const c = { title: "Homework", artist: "Daft Punk" };
            expect(isCanonicalEntityMatch(a, b)).toBe(false);
            expect(isCanonicalEntityMatch(a, c)).toBe(false);
        });

        it("matches when one or both entities lack artist if titles match", () => {
            const a = { title: "Discovery" };
            const b = { title: "Discovery", artist: "Daft Punk" };
            expect(isCanonicalEntityMatch(a, b)).toBe(true);
        });

        it("fails match when titles differ", () => {
            const a = { title: "Album 1", artist: "Artist" };
            const b = { title: "Album 2", artist: "Artist" };
            expect(isCanonicalEntityMatch(a, b)).toBe(false);
        });
    });
});
