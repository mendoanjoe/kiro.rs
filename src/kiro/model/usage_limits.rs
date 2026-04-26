//! Usage limits query data model
//!
//! Contains response type definitions for the getUsageLimits API

use serde::Deserialize;

/// Usage limits query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageLimitsResponse {
    /// Next reset date (Unix timestamp)
    #[serde(default)]
    pub next_date_reset: Option<f64>,

    /// Subscription information
    #[serde(default)]
    pub subscription_info: Option<SubscriptionInfo>,

    /// Usage breakdown list
    #[serde(default)]
    pub usage_breakdown_list: Vec<UsageBreakdown>,
}

/// Subscription information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionInfo {
    /// Subscription title (KIRO PRO+ / KIRO FREE, etc.)
    #[serde(default)]
    pub subscription_title: Option<String>,
}

/// Usage breakdown
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct UsageBreakdown {
    /// Current usage
    #[serde(default)]
    pub current_usage: i64,

    /// Current usage（精确值）
    #[serde(default)]
    pub current_usage_with_precision: f64,

    /// Bonus quota list
    #[serde(default)]
    pub bonuses: Vec<Bonus>,

    /// Free trial information
    #[serde(default)]
    pub free_trial_info: Option<FreeTrialInfo>,

    /// Next reset date (Unix timestamp)
    #[serde(default)]
    pub next_date_reset: Option<f64>,

    /// Usage limit
    #[serde(default)]
    pub usage_limit: i64,

    /// Usage limit（精确值）
    #[serde(default)]
    pub usage_limit_with_precision: f64,
}

/// Bonus quota
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bonus {
    /// Current usage
    #[serde(default)]
    pub current_usage: f64,

    /// Usage limit
    #[serde(default)]
    pub usage_limit: f64,

    /// Status (ACTIVE / EXPIRED)
    #[serde(default)]
    pub status: Option<String>,
}

impl Bonus {
    /// Check whether the bonus is active
    pub fn is_active(&self) -> bool {
        self.status
            .as_deref()
            .map(|s| s == "ACTIVE")
            .unwrap_or(false)
    }
}

/// Free trial information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FreeTrialInfo {
    /// Current usage
    #[serde(default)]
    pub current_usage: i64,

    /// Current usage（精确值）
    #[serde(default)]
    pub current_usage_with_precision: f64,

    /// Free trial expiry time (Unix timestamp)
    #[serde(default)]
    pub free_trial_expiry: Option<f64>,

    /// Free trial status (ACTIVE / EXPIRED)
    #[serde(default)]
    pub free_trial_status: Option<String>,

    /// Usage limit
    #[serde(default)]
    pub usage_limit: i64,

    /// Usage limit（精确值）
    #[serde(default)]
    pub usage_limit_with_precision: f64,
}

// ============ Convenience Method Implementations ============

impl FreeTrialInfo {
    /// Check whether the free trial is active
    pub fn is_active(&self) -> bool {
        self.free_trial_status
            .as_deref()
            .map(|s| s == "ACTIVE")
            .unwrap_or(false)
    }
}

impl UsageLimitsResponse {
    /// Get the subscription title
    pub fn subscription_title(&self) -> Option<&str> {
        self.subscription_info
            .as_ref()
            .and_then(|info| info.subscription_title.as_deref())
    }

    /// Get the first usage breakdown entry
    fn primary_breakdown(&self) -> Option<&UsageBreakdown> {
        self.usage_breakdown_list.first()
    }

    /// Get the total usage limit (precise value)
    ///
    /// Accumulates the base quota, active free-trial quota, and active bonus quota
    pub fn usage_limit(&self) -> f64 {
        let Some(breakdown) = self.primary_breakdown() else {
            return 0.0;
        };

        let mut total = breakdown.usage_limit_with_precision;

        // Accumulate active free trial quota
        if let Some(trial) = &breakdown.free_trial_info {
            if trial.is_active() {
                total += trial.usage_limit_with_precision;
            }
        }

        // Accumulate active bonus quota
        for bonus in &breakdown.bonuses {
            if bonus.is_active() {
                total += bonus.usage_limit;
            }
        }

        total
    }

    /// Get the total current usage (precise value)
    ///
    /// Accumulates the base usage, active free-trial usage, and active bonus usage
    pub fn current_usage(&self) -> f64 {
        let Some(breakdown) = self.primary_breakdown() else {
            return 0.0;
        };

        let mut total = breakdown.current_usage_with_precision;

        // Accumulate active free trial usage
        if let Some(trial) = &breakdown.free_trial_info {
            if trial.is_active() {
                total += trial.current_usage_with_precision;
            }
        }

        // Accumulate active bonus usage
        for bonus in &breakdown.bonuses {
            if bonus.is_active() {
                total += bonus.current_usage;
            }
        }

        total
    }
}
