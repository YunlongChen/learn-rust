//! Protocol extensions for domain-agent.
//!
//! This module extends the base agent protocol with diagnostic-related messages.

use serde::{Deserialize, Serialize};

/// Enhanced agent metrics with network quality indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAgentMetrics {
    /// Latency in milliseconds.
    pub latency_ms: u32,
    /// Jitter in milliseconds.
    pub jitter_ms: u32,
    /// Packet loss percentage (0-100).
    pub packet_loss_percent: f32,
    /// Bandwidth in kbps.
    pub bandwidth_kbps: u32,
}

impl Default for EnhancedAgentMetrics {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            jitter_ms: 0,
            packet_loss_percent: 0.0,
            bandwidth_kbps: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_metrics_default() {
        let metrics = EnhancedAgentMetrics::default();
        assert_eq!(metrics.latency_ms, 0);
        assert_eq!(metrics.jitter_ms, 0);
        assert_eq!(metrics.packet_loss_percent, 0.0);
        assert_eq!(metrics.bandwidth_kbps, 0);
    }
}
