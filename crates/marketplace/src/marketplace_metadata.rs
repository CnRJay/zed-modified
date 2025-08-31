use chrono::{DateTime, Utc};
use collections::BTreeSet;
use rpc::{ExtensionApiManifest, ExtensionProvides};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Enhanced extension metadata with marketplace features
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MarketplaceExtensionMetadata {
    /// Basic extension information (same as current ExtensionMetadata)
    #[serde(flatten)]
    pub manifest: ExtensionApiManifest,

    /// Unique identifier for the extension
    pub id: Arc<str>,

    /// Publication timestamp
    pub published_at: DateTime<Utc>,

    /// Total download count
    pub download_count: u64,

    /// Marketplace-specific fields
    #[serde(flatten)]
    pub marketplace_info: MarketplaceInfo,
}

/// Marketplace-specific information for extensions
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MarketplaceInfo {
    /// Average rating (1-5 stars)
    pub average_rating: Option<f32>,

    /// Total number of ratings
    pub rating_count: u32,

    /// Number of reviews
    pub review_count: u32,

    /// Categories this extension belongs to
    pub categories: BTreeSet<ExtensionCategory>,

    /// Tags for search and filtering
    pub tags: Vec<String>,

    /// Featured status (promoted extensions)
    pub featured: bool,

    /// Publisher information
    pub publisher: PublisherInfo,

    /// Gallery information (screenshots, etc.)
    pub gallery: Vec<GalleryItem>,

    /// Compatibility information
    pub compatibility: CompatibilityInfo,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Extension categories for organization
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, strum::Display, strum::EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ExtensionCategory {
    ProgrammingLanguages,
    Snippets,
    Linters,
    Formatters,
    Debuggers,
    Testing,
    Themes,
    IconThemes,
    Other,
}

/// Publisher information
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct PublisherInfo {
    /// Publisher name
    pub name: String,

    /// Publisher display name
    pub display_name: String,

    /// Publisher email (optional)
    pub email: Option<String>,

    /// Publisher website (optional)
    pub website: Option<String>,

    /// Publisher avatar URL (optional)
    pub avatar_url: Option<String>,

    /// Verified publisher status
    pub verified: bool,
}

/// Gallery item (screenshots, videos)
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct GalleryItem {
    /// Type of gallery item
    pub item_type: GalleryItemType,

    /// URL to the media
    pub url: String,

    /// Alt text for accessibility
    pub alt_text: String,

    /// Thumbnail URL (optional)
    pub thumbnail_url: Option<String>,
}

/// Type of gallery item
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum GalleryItemType {
    Screenshot,
    Video,
}

/// Compatibility information
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CompatibilityInfo {
    /// Minimum Zed version required
    pub minimum_zed_version: Option<String>,

    /// Maximum Zed version supported
    pub maximum_zed_version: Option<String>,

    /// Operating systems supported
    pub supported_os: Vec<OperatingSystem>,
}

/// Supported operating systems
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
}

/// Extension review
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ExtensionReview {
    /// Review ID
    pub id: String,

    /// Extension ID being reviewed
    pub extension_id: Arc<str>,

    /// User who wrote the review
    pub user_id: String,

    /// User display name
    pub user_display_name: String,

    /// Rating (1-5 stars)
    pub rating: u8,

    /// Review title
    pub title: String,

    /// Review content
    pub content: String,

    /// Timestamp when review was created
    pub created_at: DateTime<Utc>,

    /// Timestamp when review was last updated
    pub updated_at: DateTime<Utc>,

    /// Helpful votes count
    pub helpful_count: u32,
}

/// Extension pack (collection of extensions)
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ExtensionPack {
    /// Pack ID
    pub id: Arc<str>,

    /// Pack name
    pub name: String,

    /// Pack description
    pub description: String,

    /// Publisher information
    pub publisher: PublisherInfo,

    /// Extensions included in this pack
    pub extensions: Vec<ExtensionPackItem>,

    /// Categories for this pack
    pub categories: BTreeSet<ExtensionCategory>,

    /// Total download count
    pub download_count: u64,

    /// Average rating
    pub average_rating: Option<f32>,

    /// Rating count
    pub rating_count: u32,
}

/// Extension pack item
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ExtensionPackItem {
    /// Extension ID
    pub extension_id: Arc<str>,

    /// Extension name (for display)
    pub name: String,

    /// Version constraint (optional)
    pub version: Option<String>,
}

/// Marketplace search filters
#[derive(Clone, Debug, Default)]
pub struct MarketplaceFilters {
    /// Search query
    pub query: Option<String>,

    /// Categories to filter by
    pub categories: BTreeSet<ExtensionCategory>,

    /// Sort order
    pub sort_by: SortBy,

    /// Target provides (languages, themes, etc.)
    pub target_provides: Option<BTreeSet<ExtensionProvides>>,

    /// Minimum rating filter
    pub min_rating: Option<f32>,

    /// Publisher filter
    pub publisher: Option<String>,

    /// Featured extensions only
    pub featured_only: bool,
}

/// Sort options for marketplace search
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SortBy {
    /// Sort by relevance (default)
    #[default]
    Relevance,

    /// Sort by download count (descending)
    Downloads,

    /// Sort by average rating (descending)
    Rating,

    /// Sort by publication date (newest first)
    Newest,

    /// Sort by last updated (newest first)
    RecentlyUpdated,

    /// Sort alphabetically by name
    Name,
}

/// Marketplace search results
#[derive(Clone, Debug)]
pub struct MarketplaceSearchResults {
    /// Extensions matching the search
    pub extensions: Vec<MarketplaceExtensionMetadata>,

    /// Extension packs matching the search
    pub extension_packs: Vec<ExtensionPack>,

    /// Total number of results (for pagination)
    pub total_count: u64,

    /// Current page
    pub page: u32,

    /// Page size
    pub page_size: u32,

    /// Applied filters
    pub filters: MarketplaceFilters,
}
