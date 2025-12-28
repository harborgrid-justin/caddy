//! Marketplace store management
//!
//! This module provides store catalog management, featuring, categories,
//! and pricing tiers for the plugin marketplace.

use super::{MarketplaceError, PluginCategory, PluginMetadata, PluginStatus, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Pricing tier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PricingTier {
    /// Free plugin
    Free,

    /// One-time payment
    Premium {
        /// Price in USD cents
        price_cents: u32,
    },

    /// Subscription-based
    Subscription {
        /// Monthly price in USD cents
        monthly_price_cents: u32,

        /// Annual price in USD cents (optional discount)
        annual_price_cents: Option<u32>,
    },

    /// Enterprise licensing (contact for pricing)
    Enterprise,

    /// Pay what you want
    PayWhatYouWant {
        /// Minimum price in USD cents
        min_price_cents: u32,
    },
}

impl PricingTier {
    /// Get display name
    pub fn display_name(&self) -> String {
        match self {
            Self::Free => "Free".to_string(),
            Self::Premium { price_cents } => {
                format!("${:.2}", *price_cents as f64 / 100.0)
            }
            Self::Subscription { monthly_price_cents, .. } => {
                format!("${:.2}/month", *monthly_price_cents as f64 / 100.0)
            }
            Self::Enterprise => "Enterprise".to_string(),
            Self::PayWhatYouWant { min_price_cents } => {
                format!("Pay what you want (min ${:.2})", *min_price_cents as f64 / 100.0)
            }
        }
    }

    /// Check if tier is free
    pub fn is_free(&self) -> bool {
        matches!(self, Self::Free)
    }

    /// Get minimum price in cents
    pub fn min_price_cents(&self) -> u32 {
        match self {
            Self::Free => 0,
            Self::Premium { price_cents } => *price_cents,
            Self::Subscription { monthly_price_cents, .. } => *monthly_price_cents,
            Self::Enterprise => 0, // Contact for pricing
            Self::PayWhatYouWant { min_price_cents } => *min_price_cents,
        }
    }
}

/// Store plugin (plugin metadata + store-specific info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorePlugin {
    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Pricing tier
    pub pricing: PricingTier,

    /// Featured ranking (0 = not featured, higher = more prominent)
    pub featured_rank: u32,

    /// Category rankings
    pub category_ranks: HashMap<PluginCategory, u32>,

    /// Store-specific tags
    pub store_tags: Vec<String>,

    /// Promotional discount (percentage)
    pub discount_percent: Option<u8>,

    /// Promotion end date
    pub promotion_ends: Option<DateTime<Utc>>,

    /// Revenue share percentage (0-100)
    pub revenue_share_percent: u8,

    /// Total revenue (USD cents)
    pub total_revenue_cents: u64,

    /// Purchase count
    pub purchase_count: u64,

    /// Refund count
    pub refund_count: u64,

    /// Average support response time (hours)
    pub avg_support_response_hours: Option<f32>,

    /// Last featured date
    pub last_featured: Option<DateTime<Utc>>,

    /// Added to store date
    pub added_to_store: DateTime<Utc>,
}

impl StorePlugin {
    /// Create from plugin metadata
    pub fn from_metadata(metadata: PluginMetadata, pricing: PricingTier) -> Self {
        Self {
            metadata,
            pricing,
            featured_rank: 0,
            category_ranks: HashMap::new(),
            store_tags: Vec::new(),
            discount_percent: None,
            promotion_ends: None,
            revenue_share_percent: 70, // Default 70% to developer
            total_revenue_cents: 0,
            purchase_count: 0,
            refund_count: 0,
            avg_support_response_hours: None,
            last_featured: None,
            added_to_store: Utc::now(),
        }
    }

    /// Get effective price after discount
    pub fn effective_price_cents(&self) -> u32 {
        let base_price = self.pricing.min_price_cents();

        if let Some(discount) = self.discount_percent {
            let discount_amount = (base_price as f64 * discount as f64 / 100.0) as u32;
            base_price.saturating_sub(discount_amount)
        } else {
            base_price
        }
    }

    /// Check if currently on promotion
    pub fn is_on_promotion(&self) -> bool {
        if let Some(ends) = self.promotion_ends {
            Utc::now() < ends && self.discount_percent.is_some()
        } else {
            false
        }
    }

    /// Calculate refund rate
    pub fn refund_rate(&self) -> f32 {
        if self.purchase_count == 0 {
            0.0
        } else {
            self.refund_count as f32 / self.purchase_count as f32
        }
    }
}

/// Store catalog
#[derive(Debug)]
pub struct StoreCatalog {
    /// All store plugins (plugin_id -> store plugin)
    plugins: Arc<RwLock<HashMap<Uuid, StorePlugin>>>,

    /// Featured plugins (sorted by rank)
    featured: Arc<RwLock<Vec<Uuid>>>,

    /// Category index (category -> plugin_ids sorted by rank)
    categories: Arc<RwLock<HashMap<PluginCategory, Vec<Uuid>>>>,

    /// Tag index (tag -> plugin_ids)
    tags: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,

    /// Catalog configuration
    config: Arc<RwLock<CatalogConfig>>,
}

impl StoreCatalog {
    /// Create a new store catalog
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            featured: Arc::new(RwLock::new(Vec::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            tags: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(CatalogConfig::default())),
        }
    }

    /// Add plugin to catalog
    pub fn add_plugin(&self, store_plugin: StorePlugin) -> Result<()> {
        let plugin_id = store_plugin.metadata.manifest.id;

        // Only add published plugins
        if store_plugin.metadata.status != PluginStatus::Published {
            return Err(MarketplaceError::InvalidManifest(
                "Only published plugins can be added to store".to_string()
            ));
        }

        // Update featured list if featured
        if store_plugin.featured_rank > 0 {
            let mut featured = self.featured.write();
            if !featured.contains(&plugin_id) {
                featured.push(plugin_id);
                // Sort by rank (descending)
                featured.sort_by(|a, b| {
                    let rank_a = self.plugins.read().get(a).map(|p| p.featured_rank).unwrap_or(0);
                    let rank_b = self.plugins.read().get(b).map(|p| p.featured_rank).unwrap_or(0);
                    rank_b.cmp(&rank_a)
                });
            }
        }

        // Update category indexes
        let category = store_plugin.metadata.manifest.category;
        self.add_to_category(plugin_id, category);

        for &cat in &store_plugin.metadata.manifest.categories {
            self.add_to_category(plugin_id, cat);
        }

        // Update tag indexes
        for tag in &store_plugin.store_tags {
            self.add_to_tag(plugin_id, tag.clone());
        }

        // Add to catalog
        self.plugins.write().insert(plugin_id, store_plugin);

        Ok(())
    }

    /// Remove plugin from catalog
    pub fn remove_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let mut plugins = self.plugins.write();

        if let Some(store_plugin) = plugins.remove(&plugin_id) {
            // Remove from featured
            self.featured.write().retain(|&id| id != plugin_id);

            // Remove from categories
            let category = store_plugin.metadata.manifest.category;
            self.remove_from_category(plugin_id, category);

            for &cat in &store_plugin.metadata.manifest.categories {
                self.remove_from_category(plugin_id, cat);
            }

            // Remove from tags
            for tag in &store_plugin.store_tags {
                self.remove_from_tag(plugin_id, tag);
            }

            Ok(())
        } else {
            Err(MarketplaceError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Get plugin from catalog
    pub fn get_plugin(&self, plugin_id: Uuid) -> Option<StorePlugin> {
        self.plugins.read().get(&plugin_id).cloned()
    }

    /// Get featured plugins
    pub fn get_featured(&self, limit: usize) -> Vec<StorePlugin> {
        let featured = self.featured.read();
        let plugins = self.plugins.read();

        featured.iter()
            .take(limit)
            .filter_map(|id| plugins.get(id).cloned())
            .collect()
    }

    /// Get plugins by category
    pub fn get_by_category(&self, category: PluginCategory, limit: usize) -> Vec<StorePlugin> {
        let categories = self.categories.read();
        let plugins = self.plugins.read();

        if let Some(plugin_ids) = categories.get(&category) {
            plugin_ids.iter()
                .take(limit)
                .filter_map(|id| plugins.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get plugins by tag
    pub fn get_by_tag(&self, tag: &str, limit: usize) -> Vec<StorePlugin> {
        let tags = self.tags.read();
        let plugins = self.plugins.read();

        if let Some(plugin_ids) = tags.get(tag) {
            plugin_ids.iter()
                .take(limit)
                .filter_map(|id| plugins.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get new releases
    pub fn get_new_releases(&self, limit: usize) -> Vec<StorePlugin> {
        let mut plugins: Vec<StorePlugin> = self.plugins.read()
            .values()
            .cloned()
            .collect();

        // Sort by added_to_store date (newest first)
        plugins.sort_by(|a, b| b.added_to_store.cmp(&a.added_to_store));

        plugins.into_iter().take(limit).collect()
    }

    /// Get top rated plugins
    pub fn get_top_rated(&self, limit: usize) -> Vec<StorePlugin> {
        let mut plugins: Vec<StorePlugin> = self.plugins.read()
            .values()
            .filter(|p| p.metadata.rating_count >= self.config.read().min_ratings_for_top_rated)
            .cloned()
            .collect();

        // Sort by average rating (descending)
        plugins.sort_by(|a, b| {
            b.metadata.average_rating
                .partial_cmp(&a.metadata.average_rating)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        plugins.into_iter().take(limit).collect()
    }

    /// Get trending plugins (based on recent downloads)
    pub fn get_trending(&self, limit: usize) -> Vec<StorePlugin> {
        let mut plugins: Vec<StorePlugin> = self.plugins.read()
            .values()
            .cloned()
            .collect();

        // In production, this would be based on download velocity
        // For now, sort by download count
        plugins.sort_by(|a, b| {
            b.metadata.download_count.cmp(&a.metadata.download_count)
        });

        plugins.into_iter().take(limit).collect()
    }

    /// Get free plugins
    pub fn get_free(&self, limit: usize) -> Vec<StorePlugin> {
        let plugins = self.plugins.read();

        plugins.values()
            .filter(|p| p.pricing.is_free())
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get plugins on sale
    pub fn get_on_sale(&self, limit: usize) -> Vec<StorePlugin> {
        let plugins = self.plugins.read();

        plugins.values()
            .filter(|p| p.is_on_promotion())
            .take(limit)
            .cloned()
            .collect()
    }

    /// Search catalog
    pub fn search(&self, query: &str, limit: usize) -> Vec<StorePlugin> {
        let query_lower = query.to_lowercase();
        let plugins = self.plugins.read();

        plugins.values()
            .filter(|p| {
                p.metadata.manifest.name.to_lowercase().contains(&query_lower)
                    || p.metadata.manifest.description.to_lowercase().contains(&query_lower)
                    || p.metadata.manifest.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
                    || p.store_tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get catalog statistics
    pub fn statistics(&self) -> CatalogStatistics {
        let plugins = self.plugins.read();

        let mut stats = CatalogStatistics::default();

        for plugin in plugins.values() {
            stats.total_plugins += 1;

            match plugin.pricing {
                PricingTier::Free => stats.free_plugins += 1,
                _ => stats.paid_plugins += 1,
            }

            if plugin.featured_rank > 0 {
                stats.featured_plugins += 1;
            }

            stats.total_downloads += plugin.metadata.download_count;
            stats.total_revenue_cents += plugin.total_revenue_cents;
        }

        stats
    }

    /// Add plugin to category index
    fn add_to_category(&self, plugin_id: Uuid, category: PluginCategory) {
        let mut categories = self.categories.write();
        categories.entry(category)
            .or_insert_with(Vec::new)
            .push(plugin_id);
    }

    /// Remove plugin from category index
    fn remove_from_category(&self, plugin_id: Uuid, category: PluginCategory) {
        let mut categories = self.categories.write();
        if let Some(plugins) = categories.get_mut(&category) {
            plugins.retain(|&id| id != plugin_id);
        }
    }

    /// Add plugin to tag index
    fn add_to_tag(&self, plugin_id: Uuid, tag: String) {
        let mut tags = self.tags.write();
        tags.entry(tag)
            .or_insert_with(Vec::new)
            .push(plugin_id);
    }

    /// Remove plugin from tag index
    fn remove_from_tag(&self, plugin_id: Uuid, tag: &str) {
        let mut tags = self.tags.write();
        if let Some(plugins) = tags.get_mut(tag) {
            plugins.retain(|&id| id != plugin_id);
        }
    }
}

impl Default for StoreCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Catalog configuration
#[derive(Debug, Clone)]
struct CatalogConfig {
    /// Minimum ratings required for top rated list
    min_ratings_for_top_rated: u32,

    /// Default revenue share percentage
    default_revenue_share_percent: u8,

    /// Maximum featured plugins
    max_featured: usize,
}

impl Default for CatalogConfig {
    fn default() -> Self {
        Self {
            min_ratings_for_top_rated: 10,
            default_revenue_share_percent: 70,
            max_featured: 20,
        }
    }
}

/// Catalog statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogStatistics {
    /// Total plugins in catalog
    pub total_plugins: u64,

    /// Free plugins
    pub free_plugins: u64,

    /// Paid plugins
    pub paid_plugins: u64,

    /// Featured plugins
    pub featured_plugins: u64,

    /// Total downloads
    pub total_downloads: u64,

    /// Total revenue (USD cents)
    pub total_revenue_cents: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enterprise::marketplace::plugin::{PluginManifest, PluginMetadata};

    #[test]
    fn test_pricing_tier() {
        let free = PricingTier::Free;
        assert!(free.is_free());
        assert_eq!(free.min_price_cents(), 0);

        let premium = PricingTier::Premium { price_cents: 1999 };
        assert!(!premium.is_free());
        assert_eq!(premium.min_price_cents(), 1999);
        assert_eq!(premium.display_name(), "$19.99");
    }

    #[test]
    fn test_store_plugin_discount() {
        let manifest = PluginManifest::new("test".to_string(), "1.0.0".to_string(), "author".to_string());
        let metadata = PluginMetadata::from_manifest(manifest);
        let pricing = PricingTier::Premium { price_cents: 1000 };

        let mut store_plugin = StorePlugin::from_metadata(metadata, pricing);
        assert_eq!(store_plugin.effective_price_cents(), 1000);

        store_plugin.discount_percent = Some(20);
        store_plugin.promotion_ends = Some(Utc::now() + chrono::Duration::days(7));

        assert_eq!(store_plugin.effective_price_cents(), 800);
        assert!(store_plugin.is_on_promotion());
    }

    #[test]
    fn test_catalog_operations() -> Result<()> {
        let catalog = StoreCatalog::new();

        let manifest = PluginManifest::new("test".to_string(), "1.0.0".to_string(), "author".to_string());
        let mut metadata = PluginMetadata::from_manifest(manifest);
        metadata.status = PluginStatus::Published;

        let store_plugin = StorePlugin::from_metadata(metadata, PricingTier::Free);
        let plugin_id = store_plugin.metadata.manifest.id;

        catalog.add_plugin(store_plugin)?;

        let retrieved = catalog.get_plugin(plugin_id);
        assert!(retrieved.is_some());

        catalog.remove_plugin(plugin_id)?;

        let retrieved = catalog.get_plugin(plugin_id);
        assert!(retrieved.is_none());

        Ok(())
    }
}
