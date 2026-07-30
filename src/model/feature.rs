//! Features and categories a service, stop or vehicle may be labelled with.

use serde::{Deserialize, Serialize};

use crate::types::NaturalLanguageString;

/// A property of the service running on a line, e.g. `express` or `schoolBus`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceFeature {
    /// The code identifying the feature.
    #[serde(rename = "ServiceFeatureCode")]
    pub service_feature_code: String,
    /// Names of the feature, one per language.
    #[serde(rename = "Name")]
    pub name: Vec<NaturalLanguageString>,
    /// Icon to present the feature with.
    #[serde(rename = "Icon")]
    pub icon: String,
}

/// A commercial category a service belongs to, e.g. a fare product family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductCategory {
    /// The code identifying the category.
    #[serde(rename = "ProductCategoryCode")]
    pub product_category_code: String,
    /// Names of the category, one per language.
    #[serde(rename = "Name")]
    pub name: Vec<NaturalLanguageString>,
    /// Icon to present the category with.
    #[serde(rename = "Icon", default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// A property of a vehicle, e.g. `lowFloor` or `wheelchairAccessible`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VehicleFeature {
    /// The code identifying the feature.
    #[serde(rename = "VehicleFeatureCode")]
    pub vehicle_feature_code: String,
    /// Names of the feature, one per language.
    #[serde(rename = "Name")]
    pub name: Vec<NaturalLanguageString>,
    /// Icon to present the feature with.
    #[serde(rename = "Icon")]
    pub icon: String,
}
