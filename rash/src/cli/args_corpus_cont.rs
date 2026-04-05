/// Dataset export format
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum DatasetExportFormat {
    /// JSON array (pretty-printed)
    #[default]
    Json,
    /// JSON Lines (one object per line)
    Jsonl,
    /// CSV with headers
    Csv,
    /// Classification JSONL for ML fine-tuning ({"input":"...","label":N})
    Classification,
    /// Multi-label classification JSONL ({"input":"...","labels":[0.0, 1.0, ...]})
    MultiLabelClassification,
}

/// Corpus output format
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum CorpusOutputFormat {
    /// Human-readable report
    #[default]
    Human,
    /// JSON output
    Json,
}

/// Corpus format filter
#[derive(Clone, Debug, ValueEnum)]
pub enum CorpusFormatArg {
    /// Bash shell scripts
    Bash,
    /// Makefiles
    Makefile,
    /// Dockerfiles
    Dockerfile,
}
