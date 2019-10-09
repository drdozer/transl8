use super::feature_table::FeatureTable;


// A record of an embl-like entry.
struct Embl {
  annotations: Vec<Annotation>,
  features: FeatureTable,
  sequence: String
}

struct Annotation {
  name: String,
  values: Vec<String>,
}

