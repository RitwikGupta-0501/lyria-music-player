pub mod models;
pub mod parser;
#[cfg(test)]
pub mod tests;

pub use models::*;

use std::sync::Arc;
use std::sync::mpsc::Sender;
use tokio::sync::oneshot;
use crate::db::DbRequest;
use crate::providers::ProviderManager;

pub struct LyricsManager {
    provider_manager: Arc<ProviderManager>,
    db_tx: Sender<DbRequest>,
}

impl LyricsManager {
    pub fn new(provider_manager: Arc<ProviderManager>, db_tx: Sender<DbRequest>) -> Self {
        Self {
            provider_manager,
            db_tx,
        }
    }

    /// Generates a canonical storage key from a track's title and artist
    pub fn generate_track_key(&self, title: &str, artist: &str) -> String {
        crate::db::canonical::make_canonical_key(title, artist)
    }

    /// Resolves lyrics for a song with cache-first lookup, preferred provider routing, and automatic fallbacks.
    pub async fn get_lyrics(
        &self,
        query: &LyricsQuery,
        provider_override: Option<&str>,
    ) -> Result<Option<LyricsResult>, String> {
        let clean_title = crate::db::canonical::clean_title(&query.title);
        let clean_artist = crate::db::canonical::clean_artist(&query.artist);
        let canonical_key = self.generate_track_key(&clean_title, &clean_artist);

        // 1. Check local SQLite cache first (near-instant resolution without WASM/network)
        let (tx, rx) = oneshot::channel();
        if self.db_tx.send(DbRequest::GetCachedLyrics {
            canonical_key: canonical_key.clone(),
            provider_id: provider_override.map(|s| s.to_string()),
            resp: tx,
        }).is_ok() {
            if let Ok(Ok(Some(cached))) = rx.await {
                tracing::debug!("Lyrics cache hit for '{} - {}' [{}]", clean_artist, clean_title, cached.provider_id);
                return Ok(Some(cached));
            }
        }

        // 2. Prepare normalized query for external provider
        let normalized_query = LyricsQuery {
            title: clean_title.clone(),
            artist: clean_artist.clone(),
            album: query.album.clone(),
            duration_ms: query.duration_ms,
            isrc: query.isrc.clone(),
            track_id: query.track_id.clone(),
        };

        // 3. Determine provider execution sequence
        let candidate_providers = if let Some(override_id) = provider_override {
            vec![override_id.to_string()]
        } else {
            self.resolve_provider_cascade().await?
        };

        if candidate_providers.is_empty() {
            tracing::info!("No enabled lyrics providers available for '{} - {}'", clean_artist, clean_title);
            return Ok(None);
        }

        // 4. Waterfall through candidate providers
        for provider_id in &candidate_providers {
            tracing::debug!("Querying lyrics provider '{}' for '{} - {}'", provider_id, clean_artist, clean_title);
            match self.provider_manager.fetch_lyrics(provider_id, &normalized_query).await {
                Ok(Some(mut lyrics)) => {
                    // Populate canonical metadata if missing
                    lyrics.provider_id = provider_id.clone();

                    // Persist to SQLite cache via dedicated actor channel
                    let (save_tx, _save_rx) = oneshot::channel();
                    let _ = self.db_tx.send(DbRequest::SaveCachedLyrics {
                        canonical_key: canonical_key.clone(),
                        lyrics: lyrics.clone(),
                        resp: save_tx,
                    });

                    // Record success metric
                    let (metric_tx, _metric_rx) = oneshot::channel();
                    let _ = self.db_tx.send(DbRequest::RecordProviderSuccess {
                        provider_id: provider_id.clone(),
                        resp: metric_tx,
                    });

                    tracing::info!("Resolved lyrics from provider '{}' for '{} - {}'", provider_id, clean_artist, clean_title);
                    return Ok(Some(lyrics));
                }
                Ok(None) => {
                    tracing::debug!("Provider '{}' returned no lyrics for '{} - {}', cascading...", provider_id, clean_artist, clean_title);
                    continue;
                }
                Err(err) => {
                    tracing::warn!("Provider '{}' encountered error fetching lyrics: {}. Triggering backoff cooldown.", provider_id, err);
                    let (fail_tx, _fail_rx) = oneshot::channel();
                    let _ = self.db_tx.send(DbRequest::RecordProviderFailure {
                        provider_id: provider_id.clone(),
                        cooldown_sec: 30,
                        resp: fail_tx,
                    });
                    continue;
                }
            }
        }

        Ok(None)
    }

    /// Computes the priority-sorted fallback chain of enabled lyrics providers
    async fn resolve_provider_cascade(&self) -> Result<Vec<String>, String> {
        let (tx, rx) = oneshot::channel();
        self.db_tx.send(DbRequest::GetProviders { resp: tx }).map_err(|e| e.to_string())?;
        let all_providers = rx.await.map_err(|e| e.to_string())??;

        let (set_tx, set_rx) = oneshot::channel();
        self.db_tx.send(DbRequest::GetSetting {
            key: "preferred_lyrics_provider".to_string(),
            resp: set_tx,
        }).map_err(|e| e.to_string())?;
        let preferred_id = set_rx.await.map_err(|e| e.to_string())??;

        // Filter for enabled providers declaring the "lyrics" capability
        let mut lyrics_providers: Vec<crate::ProviderInfo> = all_providers
            .into_iter()
            .filter(|p| {
                p.status == "enabled"
                    && p.capabilities
                        .as_ref()
                        .map(|caps| caps.iter().any(|c| c.eq_ignore_ascii_case("lyrics")))
                        .unwrap_or(false)
            })
            .collect();

        // Sort: highest priority first, then alphabetically by name
        lyrics_providers.sort_by(|a, b| {
            b.priority.cmp(&a.priority).then_with(|| a.name.cmp(&b.name))
        });

        let mut cascade: Vec<String> = Vec::new();

        // If preferred provider is configured and available, place it first
        if let Some(ref pref) = preferred_id {
            if let Some(pos) = lyrics_providers.iter().position(|p| &p.id == pref) {
                let pref_provider = lyrics_providers.remove(pos);
                cascade.push(pref_provider.id);
            }
        }

        // Add remaining candidate providers in priority order
        for p in lyrics_providers {
            cascade.push(p.id);
        }

        Ok(cascade)
    }

    /// Returns metadata and preference status for all registered lyrics providers
    pub async fn get_lyrics_providers(&self) -> Result<Vec<LyricsProviderMeta>, String> {
        let (tx, rx) = oneshot::channel();
        self.db_tx.send(DbRequest::GetProviders { resp: tx }).map_err(|e| e.to_string())?;
        let all_providers = rx.await.map_err(|e| e.to_string())??;

        let (set_tx, set_rx) = oneshot::channel();
        self.db_tx.send(DbRequest::GetSetting {
            key: "preferred_lyrics_provider".to_string(),
            resp: set_tx,
        }).map_err(|e| e.to_string())?;
        let preferred_id = set_rx.await.map_err(|e| e.to_string())??;

        let metas: Vec<LyricsProviderMeta> = all_providers
            .into_iter()
            .filter(|p| {
                p.capabilities
                    .as_ref()
                    .map(|caps| caps.iter().any(|c| c.eq_ignore_ascii_case("lyrics")))
                    .unwrap_or(false)
            })
            .map(|p| {
                let is_preferred = preferred_id.as_deref() == Some(&p.id);
                LyricsProviderMeta {
                    id: p.id,
                    name: p.name,
                    version: p.version,
                    author: p.author,
                    is_preferred,
                    priority: p.priority,
                    status: p.status,
                }
            })
            .collect();

        Ok(metas)
    }

    /// Sets or clears the user's preferred lyrics provider
    pub async fn set_preferred_lyrics_provider(&self, provider_id: Option<String>) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let value = provider_id.unwrap_or_default();
        self.db_tx.send(DbRequest::SetSetting {
            key: "preferred_lyrics_provider".to_string(),
            value,
            resp: tx,
        }).map_err(|e| e.to_string())?;
        rx.await.map_err(|e| e.to_string())??;
        Ok(())
    }

    /// Clears the lyrics cache for a specific track, or all tracks if None
    pub async fn clear_cache(&self, canonical_key: Option<String>) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.db_tx.send(DbRequest::ClearLyricsCache {
            canonical_key,
            resp: tx,
        }).map_err(|e| e.to_string())?;
        rx.await.map_err(|e| e.to_string())??;
        Ok(())
    }
}
