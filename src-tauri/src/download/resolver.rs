use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::download::models::{
    DownloadQueryInput, DownloadResolutionError, DownloadSourceResult, ProviderAttemptError,
    ProviderCandidate,
};

#[derive(Debug, Clone)]
struct CircuitState {
    consecutive_failures: u32,
    tripped_until: Option<Instant>,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    cooldown_duration: Duration,
    states: RwLock<HashMap<String, CircuitState>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_duration: Duration) -> Self {
        Self {
            failure_threshold,
            cooldown_duration,
            states: RwLock::new(HashMap::new()),
        }
    }

    pub fn is_available(&self, provider_id: &str) -> bool {
        let guard = self.states.read().unwrap();
        if let Some(state) = guard.get(provider_id) {
            if let Some(tripped_until) = state.tripped_until {
                if Instant::now() < tripped_until {
                    return false;
                }
            }
        }
        true
    }

    pub fn cooldown_remaining_secs(&self, provider_id: &str) -> Option<u64> {
        let guard = self.states.read().unwrap();
        if let Some(state) = guard.get(provider_id) {
            if let Some(tripped_until) = state.tripped_until {
                let now = Instant::now();
                if now < tripped_until {
                    return Some((tripped_until - now).as_secs());
                }
            }
        }
        None
    }

    pub fn record_success(&self, provider_id: &str) {
        let mut guard = self.states.write().unwrap();
        if let Some(state) = guard.get_mut(provider_id) {
            state.consecutive_failures = 0;
            state.tripped_until = None;
        }
    }

    pub fn record_failure(&self, provider_id: &str) {
        let mut guard = self.states.write().unwrap();
        let state = guard.entry(provider_id.to_string()).or_insert(CircuitState {
            consecutive_failures: 0,
            tripped_until: None,
        });

        state.consecutive_failures += 1;
        if state.consecutive_failures >= self.failure_threshold {
            state.tripped_until = Some(Instant::now() + self.cooldown_duration);
            tracing::warn!(
                "Circuit breaker tripped for download provider '{}' after {} consecutive failures. In cooldown for {:?}",
                provider_id,
                state.consecutive_failures,
                self.cooldown_duration
            );
        }
    }

    pub fn reset_all(&self) {
        let mut guard = self.states.write().unwrap();
        guard.clear();
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(3, Duration::from_secs(60))
    }
}

pub struct DownloadResolver {
    circuit_breaker: Arc<CircuitBreaker>,
}

impl DownloadResolver {
    pub fn new(circuit_breaker: Arc<CircuitBreaker>) -> Self {
        Self { circuit_breaker }
    }

    pub fn reset_circuit_breaker(&self) {
        self.circuit_breaker.reset_all();
    }

    pub fn sort_candidates(
        &self,
        mut candidates: Vec<ProviderCandidate>,
        preferred_provider_id: Option<&str>,
    ) -> Vec<ProviderCandidate> {
        // Tag preferred candidate
        if let Some(preferred) = preferred_provider_id {
            for c in candidates.iter_mut() {
                if c.id == preferred {
                    c.is_preferred = true;
                }
            }
        }

        // Sort: preferred first, then by priority descending, then by id ascending
        candidates.sort_by(|a, b| {
            b.is_preferred
                .cmp(&a.is_preferred)
                .then_with(|| b.priority.cmp(&a.priority))
                .then_with(|| a.id.cmp(&b.id))
        });

        candidates
    }

    pub async fn resolve_with_fallback<F, Fut>(
        &self,
        candidates: Vec<ProviderCandidate>,
        preferred_provider_id: Option<&str>,
        query: &DownloadQueryInput,
        mut invoke_provider: F,
    ) -> Result<(String, DownloadSourceResult), DownloadResolutionError>
    where
        F: FnMut(String, DownloadQueryInput) -> Fut,
        Fut: std::future::Future<Output = Result<DownloadSourceResult, String>>,
    {
        if candidates.is_empty() {
            return Err(DownloadResolutionError::NoDownloadProvidersAvailable);
        }

        let ordered_candidates = self.sort_candidates(candidates, preferred_provider_id);
        let mut attempts = Vec::new();

        for candidate in ordered_candidates {
            if !self.circuit_breaker.is_available(&candidate.id) {
                let remaining = self
                    .circuit_breaker
                    .cooldown_remaining_secs(&candidate.id)
                    .unwrap_or(0);
                tracing::info!(
                    "Skipping download provider '{}' (circuit breaker open, {}s remaining)",
                    candidate.id,
                    remaining
                );
                attempts.push(ProviderAttemptError {
                    provider_id: candidate.id.clone(),
                    provider_name: candidate.name.clone(),
                    message: format!("Circuit breaker open ({}s cooldown remaining)", remaining),
                });
                continue;
            }

            tracing::info!(
                "Attempting download resolution via provider '{}' ({}) for '{} - {}'",
                candidate.id,
                candidate.name,
                query.artist,
                query.title
            );

            match invoke_provider(candidate.id.clone(), query.clone()).await {
                Ok(source) => {
                    self.circuit_breaker.record_success(&candidate.id);
                    tracing::info!(
                        "Successfully resolved download source from provider '{}': codec={:?}, bitrate={:?}kbps",
                        candidate.id,
                        source.codec,
                        source.bitrate_kbps
                    );
                    return Ok((candidate.id, source));
                }
                Err(err_msg) => {
                    tracing::warn!(
                        "Download resolution failed for provider '{}': {}",
                        candidate.id,
                        err_msg
                    );
                    self.circuit_breaker.record_failure(&candidate.id);
                    attempts.push(ProviderAttemptError {
                        provider_id: candidate.id,
                        provider_name: candidate.name,
                        message: err_msg,
                    });
                }
            }
        }

        Err(DownloadResolutionError::AllProvidersFailed { attempts })
    }
}
