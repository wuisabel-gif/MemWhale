//! Pluggable text embedding — so similarity can be *semantic*, not just lexical.
//!
//! Local-first: the default real implementation talks to **Ollama** on
//! `localhost` (e.g. `nomic-embed-text`), which is part of the user's machine —
//! no external API. When no embedder is configured (or it fails), the scorer
//! falls back to lexical term overlap, so retrieval always works offline.

/// Anything that can turn text into a vector.
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
    fn name(&self) -> &str;
}

/// Cosine similarity in [0,1] (negatives clamped to 0 — we only care about
/// "how related," not "how opposite").
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        (dot / (na.sqrt() * nb.sqrt())).clamp(0.0, 1.0)
    }
}

/// Encode an embedding as little-endian f32 bytes (for caching as a SQLite BLOB).
pub fn vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for f in v {
        out.extend_from_slice(&f.to_le_bytes());
    }
    out
}

/// Decode little-endian f32 bytes back into an embedding. Returns an empty vec
/// if the byte length isn't a multiple of 4 (corrupt/foreign blob).
pub fn bytes_to_vec(b: &[u8]) -> Vec<f32> {
    if b.len() % 4 != 0 {
        return Vec::new();
    }
    b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Local-first embedder backed by Ollama's `/api/embeddings`.
/// Run e.g. `ollama pull nomic-embed-text` first.
pub struct OllamaEmbedder {
    endpoint: String,
    model: String,
}

impl OllamaEmbedder {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            endpoint: "http://localhost:11434".into(),
            model: model.into(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }
}

impl Default for OllamaEmbedder {
    fn default() -> Self {
        Self::new("nomic-embed-text")
    }
}

impl Embedder for OllamaEmbedder {
    fn name(&self) -> &str {
        "ollama"
    }

    fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.endpoint);
        let resp = ureq::post(&url)
            .send_json(ureq::json!({ "model": self.model, "prompt": text }))
            .map_err(|e| anyhow::anyhow!("ollama request failed: {e}"))?;
        let body: serde_json::Value = resp
            .into_json()
            .map_err(|e| anyhow::anyhow!("ollama response not JSON: {e}"))?;
        let arr = body
            .get("embedding")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("ollama response missing 'embedding'"))?;
        let vec: Vec<f32> = arr
            .iter()
            .filter_map(|x| x.as_f64().map(|f| f as f32))
            .collect();
        if vec.is_empty() {
            anyhow::bail!("ollama returned an empty embedding");
        }
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_basics() {
        assert!((cosine(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
        assert_eq!(cosine(&[], &[1.0]), 0.0);
        assert_eq!(cosine(&[1.0, 2.0], &[1.0]), 0.0); // dim mismatch
    }

    #[test]
    fn vec_bytes_roundtrip() {
        let v = vec![0.0f32, 1.5, -2.25, 3.125e9, f32::MIN];
        assert_eq!(bytes_to_vec(&vec_to_bytes(&v)), v);
        assert_eq!(vec_to_bytes(&v).len(), v.len() * 4);
        // corrupt length -> empty, no panic
        assert!(bytes_to_vec(&[1, 2, 3]).is_empty());
        assert!(bytes_to_vec(&[]).is_empty());
    }
}
