//! Health scoring service for agent network health evaluation.
//!
//! Calculates health scores based on latency, jitter, packet loss, and bandwidth metrics.

use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, ColumnTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::entities::health_score::{self, ActiveModel as HealthScoreActiveModel};

/// Network health metrics used for health score calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthMetrics {
    /// Network latency in milliseconds.
    pub latency_ms: Option<f64>,
    /// Network jitter in milliseconds.
    pub jitter_ms: Option<f64>,
    /// Packet loss percentage (0-100).
    pub packet_loss_percent: Option<f64>,
    /// Bandwidth in kilobits per second.
    pub bandwidth_kbps: Option<f64>,
}

/// Component scores for individual health factors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    pub latency_score: f64,
    pub jitter_score: f64,
    pub packet_loss_score: f64,
    pub bandwidth_score: f64,
}

/// Weights for each health factor (must sum to 1.0).
const LATENCY_WEIGHT: f64 = 0.3;
const JITTER_WEIGHT: f64 = 0.2;
const PACKET_LOSS_WEIGHT: f64 = 0.4;
const BANDWIDTH_WEIGHT: f64 = 0.1;

/// Neutral score returned when a metric is unknown (None).
const NEUTRAL_SCORE: f64 = 50.0;

/// Calculates the latency score based on thresholds.
/// - <50ms = 100
/// - <100ms = 90
/// - <200ms = 70
/// - <500ms = 50
/// - <1000ms = 30
/// - >=1000ms = 10
pub fn calculate_latency_score(latency_ms: Option<f64>) -> f64 {
    match latency_ms {
        None => NEUTRAL_SCORE,
        Some(latency) => {
            if latency < 50.0 {
                100.0
            } else if latency < 100.0 {
                90.0
            } else if latency < 200.0 {
                70.0
            } else if latency < 500.0 {
                50.0
            } else if latency < 1000.0 {
                30.0
            } else {
                10.0
            }
        }
    }
}

/// Calculates the jitter score based on thresholds.
/// - <10ms = 100
/// - <30ms = 80
/// - <50ms = 60
/// - <100ms = 40
/// - >=100ms = 20
pub fn calculate_jitter_score(jitter_ms: Option<f64>) -> f64 {
    match jitter_ms {
        None => NEUTRAL_SCORE,
        Some(jitter) => {
            if jitter < 10.0 {
                100.0
            } else if jitter < 30.0 {
                80.0
            } else if jitter < 50.0 {
                60.0
            } else if jitter < 100.0 {
                40.0
            } else {
                20.0
            }
        }
    }
}

/// Calculates the packet loss score based on thresholds.
/// - <0.1% = 100
/// - <0.5% = 90
/// - <1% = 70
/// - <3% = 50
/// - <5% = 30
/// - >=5% = 10
pub fn calculate_packet_loss_score(packet_loss_percent: Option<f64>) -> f64 {
    match packet_loss_percent {
        None => NEUTRAL_SCORE,
        Some(loss) => {
            if loss < 0.1 {
                100.0
            } else if loss < 0.5 {
                90.0
            } else if loss < 1.0 {
                70.0
            } else if loss < 3.0 {
                50.0
            } else if loss < 5.0 {
                30.0
            } else {
                10.0
            }
        }
    }
}

/// Calculates the bandwidth score based on thresholds.
/// - >10000kbps = 100
/// - >5000kbps = 90
/// - >1000kbps = 70
/// - >512kbps = 50
/// - >128kbps = 30
/// - <=128kbps = 10
pub fn calculate_bandwidth_score(bandwidth_kbps: Option<f64>) -> f64 {
    match bandwidth_kbps {
        None => NEUTRAL_SCORE,
        Some(bandwidth) => {
            if bandwidth > 10000.0 {
                100.0
            } else if bandwidth > 5000.0 {
                90.0
            } else if bandwidth > 1000.0 {
                70.0
            } else if bandwidth > 512.0 {
                50.0
            } else if bandwidth > 128.0 {
                30.0
            } else {
                10.0
            }
        }
    }
}

/// Calculates the overall health score based on network metrics.
/// Returns a score from 0-100, capped at 100.
pub fn calculate_health_score(metrics: &NetworkHealthMetrics) -> u32 {
    let latency_score = calculate_latency_score(metrics.latency_ms);
    let jitter_score = calculate_jitter_score(metrics.jitter_ms);
    let packet_loss_score = calculate_packet_loss_score(metrics.packet_loss_percent);
    let bandwidth_score = calculate_bandwidth_score(metrics.bandwidth_kbps);

    let total_score = (latency_score * LATENCY_WEIGHT)
        + (jitter_score * JITTER_WEIGHT)
        + (packet_loss_score * PACKET_LOSS_WEIGHT)
        + (bandwidth_score * BANDWIDTH_WEIGHT);

    total_score.min(100.0).round() as u32
}

/// Returns the component scores for detailed health analysis.
pub fn calculate_component_scores(metrics: &NetworkHealthMetrics) -> ComponentScores {
    ComponentScores {
        latency_score: calculate_latency_score(metrics.latency_ms),
        jitter_score: calculate_jitter_score(metrics.jitter_ms),
        packet_loss_score: calculate_packet_loss_score(metrics.packet_loss_percent),
        bandwidth_score: calculate_bandwidth_score(metrics.bandwidth_kbps),
    }
}

/// Health service for recording health scores to the database.
#[derive(Clone, Debug)]
pub struct HealthService {
    db: sea_orm::DatabaseConnection,
}

impl HealthService {
    /// Creates a new HealthService with the given database connection.
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }

    /// Records a health score for an agent.
    pub async fn record_health_score(
        &self,
        agent_id: Uuid,
        metrics: &NetworkHealthMetrics,
    ) -> Result<health_score::Model, sea_orm::DbErr> {
        let overall_score = calculate_health_score(metrics) as f64;
        let component_scores = calculate_component_scores(metrics);

        let active_model = HealthScoreActiveModel {
            id: Set(Uuid::new_v4()),
            agent_id: Set(agent_id),
            scored_at: Set(Utc::now()),
            overall_score: Set(overall_score),
            latency_ms: Set(metrics.latency_ms),
            jitter_ms: Set(metrics.jitter_ms),
            packet_loss_percent: Set(metrics.packet_loss_percent),
            bandwidth_kbps: Set(metrics.bandwidth_kbps),
            component_scores: Set(Some(serde_json::to_value(component_scores).unwrap().into())),
        };

        active_model.insert(&self.db).await
    }

    /// Gets the latest health score for an agent.
    pub async fn get_latest_score(
        &self,
        agent_id: Uuid,
    ) -> Result<Option<health_score::Model>, sea_orm::DbErr> {
        use crate::storage::entities::health_score::{Entity as HealthScoreEntity, Column};

        let result = HealthScoreEntity::find()
            .filter(Column::AgentId.eq(agent_id))
            .order_by_desc(Column::ScoredAt)
            .one(&self.db)
            .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Latency score tests
    #[test]
    fn test_latency_score_excellent() {
        assert_eq!(calculate_latency_score(Some(25.0)), 100.0);
    }

    #[test]
    fn test_latency_score_good() {
        assert_eq!(calculate_latency_score(Some(75.0)), 90.0);
    }

    #[test]
    fn test_latency_score_fair() {
        assert_eq!(calculate_latency_score(Some(150.0)), 70.0);
    }

    #[test]
    fn test_latency_score_poor() {
        assert_eq!(calculate_latency_score(Some(350.0)), 50.0);
    }

    #[test]
    fn test_latency_score_very_poor() {
        assert_eq!(calculate_latency_score(Some(750.0)), 30.0);
    }

    #[test]
    fn test_latency_score_critical() {
        assert_eq!(calculate_latency_score(Some(1500.0)), 10.0);
    }

    #[test]
    fn test_latency_score_unknown() {
        assert_eq!(calculate_latency_score(None), NEUTRAL_SCORE);
    }

    // Jitter score tests
    #[test]
    fn test_jitter_score_excellent() {
        assert_eq!(calculate_jitter_score(Some(5.0)), 100.0);
    }

    #[test]
    fn test_jitter_score_good() {
        assert_eq!(calculate_jitter_score(Some(20.0)), 80.0);
    }

    #[test]
    fn test_jitter_score_fair() {
        assert_eq!(calculate_jitter_score(Some(40.0)), 60.0);
    }

    #[test]
    fn test_jitter_score_poor() {
        assert_eq!(calculate_jitter_score(Some(75.0)), 40.0);
    }

    #[test]
    fn test_jitter_score_critical() {
        assert_eq!(calculate_jitter_score(Some(150.0)), 20.0);
    }

    #[test]
    fn test_jitter_score_unknown() {
        assert_eq!(calculate_jitter_score(None), NEUTRAL_SCORE);
    }

    // Packet loss score tests
    #[test]
    fn test_packet_loss_score_excellent() {
        assert_eq!(calculate_packet_loss_score(Some(0.05)), 100.0);
    }

    #[test]
    fn test_packet_loss_score_good() {
        assert_eq!(calculate_packet_loss_score(Some(0.3)), 90.0);
    }

    #[test]
    fn test_packet_loss_score_fair() {
        assert_eq!(calculate_packet_loss_score(Some(0.7)), 70.0);
    }

    #[test]
    fn test_packet_loss_score_poor() {
        assert_eq!(calculate_packet_loss_score(Some(2.0)), 50.0);
    }

    #[test]
    fn test_packet_loss_score_very_poor() {
        assert_eq!(calculate_packet_loss_score(Some(4.0)), 30.0);
    }

    #[test]
    fn test_packet_loss_score_critical() {
        assert_eq!(calculate_packet_loss_score(Some(10.0)), 10.0);
    }

    #[test]
    fn test_packet_loss_score_unknown() {
        assert_eq!(calculate_packet_loss_score(None), NEUTRAL_SCORE);
    }

    // Bandwidth score tests
    #[test]
    fn test_bandwidth_score_excellent() {
        assert_eq!(calculate_bandwidth_score(Some(50000.0)), 100.0);
    }

    #[test]
    fn test_bandwidth_score_good() {
        assert_eq!(calculate_bandwidth_score(Some(7500.0)), 90.0);
    }

    #[test]
    fn test_bandwidth_score_fair() {
        assert_eq!(calculate_bandwidth_score(Some(2500.0)), 70.0);
    }

    #[test]
    fn test_bandwidth_score_poor() {
        assert_eq!(calculate_bandwidth_score(Some(700.0)), 50.0);
    }

    #[test]
    fn test_bandwidth_score_very_poor() {
        assert_eq!(calculate_bandwidth_score(Some(256.0)), 30.0);
    }

    #[test]
    fn test_bandwidth_score_critical() {
        assert_eq!(calculate_bandwidth_score(Some(64.0)), 10.0);
    }

    #[test]
    fn test_bandwidth_score_unknown() {
        assert_eq!(calculate_bandwidth_score(None), NEUTRAL_SCORE);
    }

    // Overall health score tests
    #[test]
    fn test_health_score_excellent() {
        let metrics = NetworkHealthMetrics {
            latency_ms: Some(25.0),
            jitter_ms: Some(5.0),
            packet_loss_percent: Some(0.05),
            bandwidth_kbps: Some(50000.0),
        };
        assert_eq!(calculate_health_score(&metrics), 100);
    }

    #[test]
    fn test_health_score_good() {
        let metrics = NetworkHealthMetrics {
            latency_ms: Some(75.0),
            jitter_ms: Some(20.0),
            packet_loss_percent: Some(0.3),
            bandwidth_kbps: Some(7500.0),
        };
        // 90*0.3 + 80*0.2 + 90*0.4 + 90*0.1 = 27 + 16 + 36 + 9 = 88
        assert_eq!(calculate_health_score(&metrics), 88);
    }

    #[test]
    fn test_health_score_fair() {
        let metrics = NetworkHealthMetrics {
            latency_ms: Some(150.0),
            jitter_ms: Some(40.0),
            packet_loss_percent: Some(0.7),
            bandwidth_kbps: Some(2500.0),
        };
        // 70*0.3 + 60*0.2 + 70*0.4 + 70*0.1 = 21 + 12 + 28 + 7 = 68
        assert_eq!(calculate_health_score(&metrics), 68);
    }

    #[test]
    fn test_health_score_with_unknowns() {
        let metrics = NetworkHealthMetrics {
            latency_ms: Some(25.0),
            jitter_ms: None,
            packet_loss_percent: Some(0.05),
            bandwidth_kbps: None,
        };
        // 100*0.3 + 50*0.2 + 100*0.4 + 50*0.1 = 30 + 10 + 40 + 5 = 85
        assert_eq!(calculate_health_score(&metrics), 85);
    }

    #[test]
    fn test_health_score_capped_at_100() {
        let metrics = NetworkHealthMetrics {
            latency_ms: Some(25.0),
            jitter_ms: Some(5.0),
            packet_loss_percent: Some(0.05),
            bandwidth_kbps: Some(50000.0),
        };
        // Even though individual scores are 100, weights ensure it's already 100
        assert_eq!(calculate_health_score(&metrics), 100);
    }

    #[test]
    fn test_component_scores() {
        let metrics = NetworkHealthMetrics {
            latency_ms: Some(75.0),
            jitter_ms: Some(20.0),
            packet_loss_percent: Some(0.3),
            bandwidth_kbps: Some(7500.0),
        };
        let scores = calculate_component_scores(&metrics);
        assert_eq!(scores.latency_score, 90.0);
        assert_eq!(scores.jitter_score, 80.0);
        assert_eq!(scores.packet_loss_score, 90.0);
        assert_eq!(scores.bandwidth_score, 90.0);
    }
}
