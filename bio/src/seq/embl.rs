use super::feature_table::FeatureTable;


// A record of an embl-like entry.
pub struct Embl {
  pub annotations: Vec<Annotation>,
  pub features: FeatureTable,
  pub sequence: String
}

pub struct Annotation {
  pub name: String,
  pub values: Vec<String>,
}

