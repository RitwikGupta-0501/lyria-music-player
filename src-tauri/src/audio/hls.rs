use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;
use symphonia::core::io::MediaSource;
use url::Url;

pub struct HlsStream {
    client: reqwest::Client,
    runtime: tauri::async_runtime::RuntimeHandle,
    _playlist_url: Url,
    segments: Vec<Segment>,
    current_segment_idx: usize,
    current_segment_data: Option<std::io::Cursor<bytes::Bytes>>,
}

struct Segment {
    url: Url,
    duration: f32,
}

impl HlsStream {
    pub fn new(
        url: Url,
        client: reqwest::Client,
        runtime: tauri::async_runtime::RuntimeHandle,
        seek_to: Option<Duration>,
    ) -> Result<Self, String> {
        let segments = runtime.block_on(async {
            Self::fetch_all_segments(&client, url.clone()).await
        })?;

        let mut start_idx = 0;
        if let Some(pos) = seek_to {
            let pos_secs = pos.as_secs_f64();
            let mut acc: f64 = 0.0;
            for (i, seg) in segments.iter().enumerate() {
                if acc + (seg.duration as f64) > pos_secs {
                    start_idx = i;
                    break;
                }
                acc += seg.duration as f64;
            }
        }

        Ok(Self {
            client,
            runtime,
            _playlist_url: url,
            segments,
            current_segment_idx: start_idx,
            current_segment_data: None,
        })
    }

    async fn fetch_all_segments(client: &reqwest::Client, mut url: Url) -> Result<Vec<Segment>, String> {
        let mut segments = Vec::new();
        // Allow up to 3 redirects/playlist layers (e.g., Master -> Media -> Media)
        for _ in 0..3 {
            let res = client.get(url.clone()).send().await.map_err(|e| e.to_string())?;
            let body = res.bytes().await.map_err(|e| e.to_string())?;
            
            match m3u8_rs::parse_playlist_res(&body) {
                Ok(m3u8_rs::Playlist::MasterPlaylist(pl)) => {
                    // Pick the highest bandwidth variant, or fall back to the first one
                    let variant = pl.variants
                        .iter()
                        .max_by_key(|v| v.bandwidth)
                        .or_else(|| pl.variants.first())
                        .ok_or("No variants in master playlist")?;
                    url = url.join(&variant.uri).map_err(|e| e.to_string())?;
                }
                Ok(m3u8_rs::Playlist::MediaPlaylist(pl)) => {
                    for seg in pl.segments {
                        let seg_url = url.join(&seg.uri).map_err(|e| e.to_string())?;
                        segments.push(Segment {
                            url: seg_url,
                            duration: seg.duration,
                        });
                    }
                    return Ok(segments);
                }
                Err(e) => {
                    return Err(format!("Failed to parse HLS playlist: {:?}", e));
                }
            }
        }
        Err("Too many HLS playlist layers".to_string())
    }

    fn fetch_next_segment(&mut self) -> std::io::Result<()> {
        if self.current_segment_idx >= self.segments.len() {
            return Ok(()); // EOF
        }

        let seg_url = self.segments[self.current_segment_idx].url.clone();
        let client = self.client.clone();
        
        let body = self.runtime.block_on(async {
            let res = client.get(seg_url).send().await.map_err(|e| std::io::Error::other(e.to_string()))?;
            res.bytes().await.map_err(|e| std::io::Error::other(e.to_string()))
        })?;

        self.current_segment_data = Some(std::io::Cursor::new(body));
        self.current_segment_idx += 1;
        Ok(())
    }
}

impl Read for HlsStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if let Some(cursor) = &mut self.current_segment_data {
                let n = cursor.read(buf)?;
                if n > 0 {
                    return Ok(n);
                }
                // EOF for this segment, need the next one
                self.current_segment_data = None;
            }

            if self.current_segment_idx >= self.segments.len() {
                return Ok(0); // True EOF
            }

            self.fetch_next_segment()?;
        }
    }
}

impl Seek for HlsStream {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "HLS stream seeking is handled by time, not byte offset"))
    }
}

impl MediaSource for HlsStream {
    fn is_seekable(&self) -> bool {
        false // Tell Symphonia not to use byte-seeking
    }
    fn byte_len(&self) -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_master_playlist_highest_bandwidth_selection() {
        let manifest = b"#EXTM3U\n\
#EXT-X-STREAM-INF:BANDWIDTH=64000\nlow.m3u8\n\
#EXT-X-STREAM-INF:BANDWIDTH=256000\nhigh.m3u8\n\
#EXT-X-STREAM-INF:BANDWIDTH=128000\nmid.m3u8\n";

        let parsed = m3u8_rs::parse_playlist_res(manifest).expect("valid master playlist");
        match parsed {
            m3u8_rs::Playlist::MasterPlaylist(pl) => {
                let variant = pl.variants
                    .iter()
                    .max_by_key(|v| v.bandwidth)
                    .or_else(|| pl.variants.first())
                    .expect("at least one variant");
                assert_eq!(variant.bandwidth, 256000);
                assert_eq!(variant.uri, "high.m3u8");
            }
            _ => panic!("Expected MasterPlaylist"),
        }
    }
}

