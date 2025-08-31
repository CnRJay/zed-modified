//! Extension registry for caching marketplace metadata locally

use crate::marketplace_metadata::*;
use anyhow::{Context, Result};
use fs::Fs;
use gpui::App;
use rope::Rope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Local extension registry for caching marketplace data
pub struct ExtensionRegistry {
    fs: Arc<dyn Fs>,
    cache_dir: PathBuf,
    cache: RegistryCache,
    cache_dirty: bool,
}

/// Cached registry data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegistryCache {
    /// Cached extensions
    pub extensions: HashMap<String, CachedExtension>,

    /// Cached extension packs
    pub extension_packs: HashMap<String, CachedExtensionPack>,

    /// Categories and their extension counts
    pub categories: HashMap<ExtensionCategory, u32>,

    /// Popular extensions (top N by downloads)
    pub popular_extensions: Vec<String>,

    /// Featured extensions
    pub featured_extensions: Vec<String>,

    /// Last update timestamp
    pub last_updated: Option<SystemTime>,

    /// Cache version for invalidation
    pub version: u32,
}

/// Cached extension data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedExtension {
    /// Full extension metadata
    pub metadata: MarketplaceExtensionMetadata,

    /// Cache timestamp
    pub cached_at: SystemTime,

    /// Time-to-live in seconds
    pub ttl: u64,
}

/// Cached extension pack data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedExtensionPack {
    /// Full extension pack metadata
    pub metadata: ExtensionPack,

    /// Cache timestamp
    pub cached_at: SystemTime,

    /// Time-to-live in seconds
    pub ttl: u64,
}

impl ExtensionRegistry {
    /// Create a new extension registry
    pub fn new(fs: Arc<dyn Fs>, cache_dir: PathBuf) -> Self {
        Self {
            fs,
            cache_dir,
            cache: RegistryCache::default(),
            cache_dirty: false,
        }
    }

    /// Initialize the registry by loading from disk
    pub async fn initialize(&mut self) -> Result<()> {
        let cache_path = self.cache_dir.join("marketplace_cache.json");

        if self.fs.is_file(&cache_path).await {
            let cache_data = self.fs.load(&cache_path).await?;
            self.cache = serde_json::from_str(&cache_data)
                .context("Failed to parse marketplace cache")?;
        }

        Ok(())
    }

    /// Save the registry cache to disk
    pub async fn save(&mut self) -> Result<()> {
        if !self.cache_dirty {
            return Ok(());
        }

        self.cache.last_updated = Some(SystemTime::now());

        // Ensure cache directory exists
        self.fs
            .create_dir(&self.cache_dir)
            .await
            .context("Failed to create cache directory")?;

        let cache_path = self.cache_dir.join("marketplace_cache.json");
        let cache_data = serde_json::to_vec_pretty(&self.cache)
            .context("Failed to serialize cache")?;

        let cache_string = String::from_utf8(cache_data)
            .context("Failed to convert cache data to string")?;
        let cache_rope = Rope::from(cache_string);

        self.fs
            .save(&cache_path, &cache_rope, Default::default())
            .await
            .context("Failed to save cache")?;

        self.cache_dirty = false;
        Ok(())
    }

    /// Get an extension from cache if it's still valid
    pub fn get_extension(&self, extension_id: &str) -> Option<&MarketplaceExtensionMetadata> {
        self.cache.extensions.get(extension_id).and_then(|cached| {
            if self.is_cache_entry_valid(cached) {
                Some(&cached.metadata)
            } else {
                None
            }
        })
    }

    /// Cache an extension
    pub fn cache_extension(&mut self, extension: MarketplaceExtensionMetadata) {
        let cached = CachedExtension {
            metadata: extension,
            cached_at: SystemTime::now(),
            ttl: 3600, // 1 hour TTL
        };

        let extension_id = cached.metadata.id.to_string();
        self.cache.extensions.insert(extension_id, cached);
        self.cache_dirty = true;
    }

    /// Get multiple extensions from cache
    pub fn get_extensions(&self, extension_ids: &[String]) -> Vec<&MarketplaceExtensionMetadata> {
        extension_ids
            .iter()
            .filter_map(|id| self.get_extension(id))
            .collect()
    }

    /// Cache multiple extensions
    pub fn cache_extensions(&mut self, extensions: Vec<MarketplaceExtensionMetadata>) {
        for extension in extensions {
            self.cache_extension(extension);
        }
    }

    /// Get an extension pack from cache
    pub fn get_extension_pack(&self, pack_id: &str) -> Option<&ExtensionPack> {
        self.cache.extension_packs.get(pack_id).and_then(|cached| {
            if self.is_cache_entry_valid_pack(cached) {
                Some(&cached.metadata)
            } else {
                None
            }
        })
    }

    /// Cache an extension pack
    pub fn cache_extension_pack(&mut self, pack: ExtensionPack) {
        let cached = CachedExtensionPack {
            metadata: pack,
            cached_at: SystemTime::now(),
            ttl: 3600, // 1 hour TTL
        };

        let pack_id = cached.metadata.id.to_string();
        self.cache.extension_packs.insert(pack_id, cached);
        self.cache_dirty = true;
    }

    /// Get popular extensions
    pub fn get_popular_extensions(&self, limit: usize) -> Vec<&MarketplaceExtensionMetadata> {
        self.cache
            .popular_extensions
            .iter()
            .take(limit)
            .filter_map(|id| self.get_extension(id))
            .collect()
    }

    /// Set popular extensions
    pub fn set_popular_extensions(&mut self, extension_ids: Vec<String>) {
        self.cache.popular_extensions = extension_ids;
        self.cache_dirty = true;
    }

    /// Get featured extensions
    pub fn get_featured_extensions(&self) -> Vec<&MarketplaceExtensionMetadata> {
        self.cache
            .featured_extensions
            .iter()
            .filter_map(|id| self.get_extension(id))
            .collect()
    }

    /// Set featured extensions
    pub fn set_featured_extensions(&mut self, extension_ids: Vec<String>) {
        self.cache.featured_extensions = extension_ids;
        self.cache_dirty = true;
    }

    /// Search extensions in cache
    pub fn search_extensions(
        &self,
        query: Option<&str>,
        filters: &MarketplaceFilters,
    ) -> Vec<&MarketplaceExtensionMetadata> {
        let mut results: Vec<&MarketplaceExtensionMetadata> = self
            .cache
            .extensions
            .values()
            .filter(|cached| self.is_cache_entry_valid(cached))
            .map(|cached| &cached.metadata)
            .collect();

        // Apply search query filter
        if let Some(query) = query {
            let query_lower = query.to_lowercase();
            results.retain(|ext| {
                ext.manifest.name.to_lowercase().contains(&query_lower)
                    || ext.manifest.description
                        .as_ref()
                        .map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
                    || ext.marketplace_info.publisher.name.to_lowercase().contains(&query_lower)
                    || ext.marketplace_info.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            });
        }

        // Apply category filter
        if !filters.categories.is_empty() {
            results.retain(|ext| {
                ext.marketplace_info.categories.iter().any(|cat| filters.categories.contains(cat))
            });
        }

        // Apply provides filter
        if let Some(provides) = &filters.target_provides {
            results.retain(|ext| {
                ext.manifest.provides.iter().any(|provide| provides.contains(provide))
            });
        }

        // Apply rating filter
        if let Some(min_rating) = filters.min_rating {
            results.retain(|ext| {
                ext.marketplace_info.average_rating.map_or(false, |rating| rating >= min_rating)
            });
        }

        // Apply publisher filter
        if let Some(publisher) = &filters.publisher {
            results.retain(|ext| {
                ext.marketplace_info.publisher.name.to_lowercase().contains(&publisher.to_lowercase())
            });
        }

        // Apply featured filter
        if filters.featured_only {
            results.retain(|ext| ext.marketplace_info.featured);
        }

        // Sort results
        self.sort_extensions(&mut results, filters.sort_by.clone());

        results
    }

    /// Sort extensions by the given criteria
    fn sort_extensions(&self, extensions: &mut Vec<&MarketplaceExtensionMetadata>, sort_by: SortBy) {
        extensions.sort_by(|a, b| match sort_by {
            SortBy::Relevance => {
                // For relevance, prefer featured extensions, then by rating/downloads
                let a_featured = a.marketplace_info.featured as i32;
                let b_featured = b.marketplace_info.featured as i32;
                let featured_cmp = b_featured.cmp(&a_featured); // Featured first

                if featured_cmp != std::cmp::Ordering::Equal {
                    return featured_cmp;
                }

                // Then by rating
                let a_rating = a.marketplace_info.average_rating.unwrap_or(0.0);
                let b_rating = b.marketplace_info.average_rating.unwrap_or(0.0);
                b_rating.partial_cmp(&a_rating).unwrap_or(std::cmp::Ordering::Equal)
            }
            SortBy::Downloads => b.download_count.cmp(&a.download_count),
            SortBy::Rating => {
                let a_rating = a.marketplace_info.average_rating.unwrap_or(0.0);
                let b_rating = b.marketplace_info.average_rating.unwrap_or(0.0);
                b_rating.partial_cmp(&a_rating).unwrap_or(std::cmp::Ordering::Equal)
            }
            SortBy::Newest => b.published_at.cmp(&a.published_at),
            SortBy::RecentlyUpdated => b.marketplace_info.last_updated.cmp(&a.marketplace_info.last_updated),
            SortBy::Name => a.manifest.name.cmp(&b.manifest.name),
        });
    }

    /// Get category counts
    pub fn get_category_counts(&self) -> HashMap<ExtensionCategory, u32> {
        let mut counts = HashMap::new();

        for cached in self.cache.extensions.values() {
            if self.is_cache_entry_valid(cached) {
                for category in &cached.metadata.marketplace_info.categories {
                    *counts.entry(*category).or_insert(0) += 1;
                }
            }
        }

        counts
    }

    /// Update category counts in cache
    pub fn update_category_counts(&mut self) {
        self.cache.categories = self.get_category_counts();
        self.cache_dirty = true;
    }

    /// Clear expired cache entries
    pub fn clear_expired(&mut self) {
        let now = SystemTime::now();

        self.cache.extensions.retain(|_, cached| {
            if let Ok(duration) = now.duration_since(cached.cached_at) {
                duration.as_secs() < cached.ttl
            } else {
                false
            }
        });

        self.cache.extension_packs.retain(|_, cached| {
            if let Ok(duration) = now.duration_since(cached.cached_at) {
                duration.as_secs() < cached.ttl
            } else {
                false
            }
        });

        if !self.cache.extensions.is_empty() || !self.cache.extension_packs.is_empty() {
            self.cache_dirty = true;
        }
    }

    /// Clear all cache data
    pub fn clear_cache(&mut self) {
        self.cache = RegistryCache::default();
        self.cache.version += 1;
        self.cache_dirty = true;
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            extension_count: self.cache.extensions.len(),
            extension_pack_count: self.cache.extension_packs.len(),
            last_updated: self.cache.last_updated,
            cache_size_bytes: self.estimate_cache_size(),
        }
    }

    /// Estimate cache size in bytes
    fn estimate_cache_size(&self) -> u64 {
        // Rough estimation based on typical JSON sizes
        let extensions_size = self.cache.extensions.len() as u64 * 2048; // ~2KB per extension
        let packs_size = self.cache.extension_packs.len() as u64 * 1024; // ~1KB per pack
        extensions_size + packs_size
    }

    /// Check if a cache entry is still valid
    fn is_cache_entry_valid(&self, cached: &CachedExtension) -> bool {
        if let Ok(duration) = SystemTime::now().duration_since(cached.cached_at) {
            duration.as_secs() < cached.ttl
        } else {
            false
        }
    }

    /// Check if a pack cache entry is still valid
    fn is_cache_entry_valid_pack(&self, cached: &CachedExtensionPack) -> bool {
        if let Ok(duration) = SystemTime::now().duration_since(cached.cached_at) {
            duration.as_secs() < cached.ttl
        } else {
            false
        }
    }
}

/// Cache statistics
#[derive(Clone, Debug)]
pub struct CacheStats {
    /// Number of cached extensions
    pub extension_count: usize,

    /// Number of cached extension packs
    pub extension_pack_count: usize,

    /// Last cache update timestamp
    pub last_updated: Option<SystemTime>,

    /// Estimated cache size in bytes
    pub cache_size_bytes: u64,
}

impl CacheStats {
    /// Format cache size as human readable string
    pub fn format_cache_size(&self) -> String {
        if self.cache_size_bytes < 1024 {
            format!("{} bytes", self.cache_size_bytes)
        } else if self.cache_size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.cache_size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.cache_size_bytes as f64 / (1024.0 * 1024.0))
        }
    }

    /// Get time since last update as human readable string
    pub fn format_time_since_update(&self) -> String {
        if let Some(last_updated) = self.last_updated {
            if let Ok(duration) = SystemTime::now().duration_since(last_updated) {
                if duration.as_secs() < 60 {
                    format!("{} seconds ago", duration.as_secs())
                } else if duration.as_secs() < 3600 {
                    format!("{} minutes ago", duration.as_secs() / 60)
                } else if duration.as_secs() < 86400 {
                    format!("{} hours ago", duration.as_secs() / 3600)
                } else {
                    format!("{} days ago", duration.as_secs() / 86400)
                }
            } else {
                "in the future".to_string()
            }
        } else {
            "never".to_string()
        }
    }
}
