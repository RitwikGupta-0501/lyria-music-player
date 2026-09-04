# Editorial Spotlight Section — Architecture & Extension Specification

This document details the design philosophy, runtime behavior, IPC contracts, and extension capabilities required for the **Editorial Spotlight** hero banner on Echo's Explore page.

---

## 1. What is the Editorial Spotlight?

The **Editorial Spotlight** is the crown hero showcase at the top of Echo's Explore view. 

Unlike standard grid cards, the Spotlight is designed as an **immersive, widescreen narrative billboard** ($280\\text{px}$ height) that highlights 1–4 landmark musical releases, high-fidelity album premieres, or major curated collections.

### Key Visual & Interactive Features:
1. **Dynamic Ambient Light Backdrop**:
   * The high-resolution album artwork is projected into a blurred, ultra-wide gradient background with smooth cubic opacity curves.
2. **Typography & Badge Layer**:
   * Amber Sparkle badge (`EDITORIAL SPOTLIGHT`), large Serif title, artist byline, and custom release description tags (e.g., *"Featured Release • Lossless Master Edition"*).
3. **Dual Action Controls**:
   * **Play Album**: Immediate one-click playback of the entire release.
   * **Explore Release**: Opens the album's right-hand detail drawer (`CollectionDetail.svelte`) showing the complete tracklist, duration, and artist credits.
4. **Auto-Rotating Carousel Deck**:
   * Smooth 7-second auto-rotation interval with interactive pill indicators and prev/next arrow controls. Auto-rotation automatically pauses when the user hovers over the card.

---

## 2. Architecture & Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. Extension Manifest Declaration                               │
│    Declares module capability: `id: "spotlight"`                  │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Sandboxed Extism WASM Call: `fetch_module("spotlight")`       │
│    (Polyglot: Written in TypeScript, Go, Rust, Python, etc.)    │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Host Output Quality Gate & Sanitization                      │
│    Filters empty IDs, invalid thumbnails, and user spam         │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Svelte 5 Explore Store (`exploreStore.spotlights`)           │
│    Manages active slide index, 7s interval, and zero-CLS skeleton│
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Hero Component (`SpotlightCarousel.svelte`)                  │
│    Renders ambient backdrop, squircle art, and action buttons   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Extism Polyglot WASM ABI (Universal Contract)

Echo extensions are compiled to **WebAssembly (`.wasm`) using WASI**. 

Because Echo uses **[Extism](https://extism.org/)**, extensions can be written in **any programming language** (TypeScript, Go, Rust, Python, C/C++, Zig). 

The boundary between Echo and the plugin is **strictly standard JSON strings over WASI memory buffers**. The host simply invokes the exported function `fetch_module` with the module ID as JSON input:

```
Host Input:  {"id": "spotlight"}
Plugin Out:  {"items": [{"type": "Spotlight", "data": { ... }}]}
```

---

## 4. Multi-Language Plugin Implementation Examples

### 1. TypeScript / JavaScript (`@extism/js-pdk`)

```typescript
// index.ts — Compile with: extism-js index.ts -i index.d.ts -o plugin.wasm
declare const Host: any;

interface ModuleRequest {
    id: string;
}

export function fetch_module() {
    const input: ModuleRequest = JSON.parse(Host.inputString());

    if (input.id === "spotlight") {
        const payload = {
            items: [
                {
                    type: "Spotlight",
                    data: {
                        id: "MPREb_casiopea_mint_jams",
                        title: "Mint Jams",
                        artist: "Casiopea",
                        cover_art_url: "https://example.com/mint_jams.jpg",
                        description: "Live Tokyo Jazz Fusion Masterwork",
                        release_year: "1982",
                        track_count: 8
                    }
                },
                {
                    type: "Spotlight",
                    data: {
                        id: "MPREb_daft_punk_ram",
                        title: "Random Access Memories",
                        artist: "Daft Punk",
                        cover_art_url: "https://example.com/ram.jpg",
                        description: "Lossless Master Edition • Electronic Classic",
                        release_year: "2013",
                        track_count: 13
                    }
                }
            ]
        };

        Host.outputString(JSON.stringify(payload));
        return 0;
    }

    Host.outputString(JSON.stringify({ items: [] }));
    return 0;
}
```

---

### 2. Go (`tinygo` + `github.com/extism/go-pdk`)

```go
// main.go — Compile with: tinygo build -target=wasi -o plugin.wasm main.go
package main

import (
    "encoding/json"
    "github.com/extism/go-pdk"
)

type SpotlightData struct {
    ID          string `json:"id"`
    Title       string `json:"title"`
    Artist      string `json:"artist"`
    CoverArtURL string `json:"cover_art_url,omitempty"`
    Description string `json:"description,omitempty"`
    ReleaseYear string `json:"release_year,omitempty"`
    TrackCount  uint32 `json:"track_count,omitempty"`
}

type ModuleItem struct {
    Type string        `json:"type"`
    Data SpotlightData `json:"data"`
}

type ModuleData struct {
    Items []ModuleItem `json:"items"`
}

//export fetch_module
func fetch_module() int32 {
    inputBytes := pdk.Input()
    var req struct{ ID string `json:"id"` }
    _ = json.Unmarshal(inputBytes, &req)

    if req.ID == "spotlight" {
        resp := ModuleData{
            Items: []ModuleItem{
                {
                    Type: "Spotlight",
                    Data: SpotlightData{
                        ID:          "MPREb_casiopea_mint_jams",
                        Title:       "Mint Jams",
                        Artist:      "Casiopea",
                        CoverArtURL: "https://example.com/mint_jams.jpg",
                        Description: "Live Tokyo Jazz Fusion Masterwork",
                        ReleaseYear: "1982",
                        TrackCount:  8,
                    },
                },
            },
        }
        out, _ := json.Marshal(resp)
        pdk.Output(out)
        return 0
    }

    pdk.Output([]byte("{\"items\":[]}"))
    return 0
}

func main() {}
```

---

### 3. Rust (`extism-pdk` + `wasm32-wasip1`)

```rust
// lib.rs — Compile with: cargo build --target wasm32-wasip1 --release
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EditorialSpotlight {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover_art_url: Option<String>,
    pub description: Option<String>,
    pub release_year: Option<String>,
    pub track_count: Option<u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ModuleItem {
    Spotlight(EditorialSpotlight),
}

#[derive(Serialize, Deserialize)]
pub struct ModuleData {
    pub items: Vec<ModuleItem>,
}

#[plugin_fn]
pub fn fetch_module(input: String) -> FnResult<String> {
    if input.contains("spotlight") {
        let data = ModuleData {
            items: vec![
                ModuleItem::Spotlight(EditorialSpotlight {
                    id: "MPREb_casiopea_mint_jams".into(),
                    title: "Mint Jams".into(),
                    artist: "Casiopea".into(),
                    cover_art_url: Some("https://example.com/mint_jams.jpg".into()),
                    description: Some("Live Tokyo Jazz Fusion Masterwork".into()),
                    release_year: Some("1982".into()),
                    track_count: Some(8),
                })
            ]
        };
        return Ok(serde_json::to_string(&data)?);
    }
    Ok("{\"items\":[]}".to_string())
}
```

---

### 4. Python (`extism` Python PDK)

```python
# plugin.py — Compile using componentize-py or Extism Python SDK
import json
import extism

@extism.plugin_fn
def fetch_module(input_str: str) -> str:
    req = json.loads(input_str)
    
    if req.get("id") == "spotlight":
        payload = {
            "items": [
                {
                    "type": "Spotlight",
                    "data": {
                        "id": "MPREb_casiopea_mint_jams",
                        "title": "Mint Jams",
                        "artist": "Casiopea",
                        "cover_art_url": "https://example.com/mint_jams.jpg",
                        "description": "Live Tokyo Jazz Fusion Masterwork",
                        "release_year": "1982",
                        "track_count": 8
                    }
                }
            ]
        }
        return json.dumps(payload)
        
    return json.dumps({"items": []})
```

---

## 5. Universal Wire JSON Schema Specification

Regardless of the programming language chosen, the extension must output valid JSON matching this schema:

```json
{
  "items": [
    {
      "type": "Spotlight",
      "data": {
        "id": "string (Required: Unique album/playlist browse ID)",
        "title": "string (Required: Album/collection title)",
        "artist": "string (Required: Artist or curator byline)",
        "cover_art_url": "string (Optional: URL for high-res cover artwork)",
        "description": "string (Optional: Editorial summary or badge tagline)",
        "release_year": "string (Optional: Release year, e.g. \"2026\")",
        "track_count": "number (Optional: Total track count integer)"
      }
    }
  ]
}
```

---

## 6. Svelte 5 Frontend Consumer Model

```typescript
// In src/lib/stores/explore.svelte.ts
export interface SpotlightItem {
    id: string;
    title: string;
    artist: string;
    cover_art_url?: string;
    description?: string;
    release_year?: string;
    track_count?: number;
    provider_id?: string;
}
```

---

## 7. Lifecycle & Error Resilience

### 1. Zero-CLS Skeleton Loading
While the plugin executes:
* `SpotlightCarousel.svelte` displays the geometric ambient hero skeleton (`min-height: 280px`), rendering a pulsating eyebrow badge, dual title/subtitle lines, twin action pills, and a $170\text{px}$ squircle art box.
* Prevents layout popping when real data arrives.

### 2. Seamless 0px Collapse
* If an active extension returns 0 spotlight items (or encounters an error), the entire section unmounts and collapses to $0\text{px}$ without displaying orphaned headers or empty containers.
