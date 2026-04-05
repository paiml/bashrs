impl CorpusRunner {

    fn compute_format_scores(
        &self,
        results: &[CorpusResult],
        registry: &CorpusRegistry,
    ) -> Vec<FormatScore> {
        let mut scores = Vec::new();

        // KAIZEN-075: O(1) format lookup instead of O(n) linear search per result.
        // Was ~322M string comparisons per format (17,942^2 x 3 = ~966M total).
        let format_by_id: HashMap<&str, CorpusFormat> = registry
            .entries
            .iter()
            .map(|e| (e.id.as_str(), e.format))
            .collect();

        for format in &[
            CorpusFormat::Bash,
            CorpusFormat::Makefile,
            CorpusFormat::Dockerfile,
        ] {
            let format_results: Vec<&CorpusResult> = results
                .iter()
                .filter(|r| format_by_id.get(r.id.as_str()) == Some(format))
                .collect();

            if format_results.is_empty() {
                continue;
            }

            let ft = format_results.len();
            let fp = format_results.iter().filter(|r| r.transpiled).count();
            let fr = if ft > 0 { fp as f64 / ft as f64 } else { 0.0 };
            let fs = if ft > 0 {
                let ts: f64 = format_results.iter().map(|r| r.score()).sum();
                ts / ft as f64
            } else {
                0.0
            };

            scores.push(FormatScore {
                format: *format,
                total: ft,
                passed: fp,
                rate: fr,
                score: fs,
                grade: Grade::from_score(fs),
            });
        }

        scores
    }

    /// Generate a convergence entry for logging.
    pub fn convergence_entry(
        &self,
        score: &CorpusScore,
        iteration: u32,
        date: &str,
        previous_rate: f64,
        notes: &str,
    ) -> ConvergenceEntry {
        // Extract per-format stats from format_scores (spec SS11.10.5)
        let (bash_passed, bash_total) = score
            .format_score(CorpusFormat::Bash)
            .map_or((0, 0), |fs| (fs.passed, fs.total));
        let (makefile_passed, makefile_total) = score
            .format_score(CorpusFormat::Makefile)
            .map_or((0, 0), |fs| (fs.passed, fs.total));
        let (dockerfile_passed, dockerfile_total) = score
            .format_score(CorpusFormat::Dockerfile)
            .map_or((0, 0), |fs| (fs.passed, fs.total));

        let bash_score = score
            .format_score(CorpusFormat::Bash)
            .map_or(0.0, |fs| fs.score);
        let makefile_score = score
            .format_score(CorpusFormat::Makefile)
            .map_or(0.0, |fs| fs.score);
        let dockerfile_score = score
            .format_score(CorpusFormat::Dockerfile)
            .map_or(0.0, |fs| fs.score);

        let lint_passed = score.results.iter().filter(|r| r.lint_clean).count();
        let lint_rate = if score.total > 0 {
            lint_passed as f64 / score.total as f64
        } else {
            0.0
        };

        ConvergenceEntry {
            iteration,
            date: date.to_string(),
            total: score.total,
            passed: score.passed,
            failed: score.failed,
            rate: score.rate,
            delta: score.rate - previous_rate,
            notes: notes.to_string(),
            bash_passed,
            bash_total,
            makefile_passed,
            makefile_total,
            dockerfile_passed,
            dockerfile_total,
            score: score.score,
            grade: score.grade.to_string(),
            bash_score,
            makefile_score,
            dockerfile_score,
            lint_passed,
            lint_rate,
        }
    }

    /// Append a convergence entry to a JSONL log file.
    /// Creates parent directories if needed.
    pub fn append_convergence_log(
        entry: &ConvergenceEntry,
        path: &std::path::Path,
    ) -> std::io::Result<()> {
        use std::io::Write;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let json = serde_json::to_string(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        writeln!(file, "{json}")?;
        Ok(())
    }

    /// Load convergence entries from a JSONL log file.
    /// Returns empty vec if file does not exist.
    pub fn load_convergence_log(path: &std::path::Path) -> std::io::Result<Vec<ConvergenceEntry>> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut entries = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: ConvergenceEntry = serde_json::from_str(trimmed)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Check convergence criteria: rate >= 99% for 3 consecutive iterations,
    /// delta < 0.5% for 3 consecutive iterations.
    pub fn is_converged(entries: &[ConvergenceEntry]) -> bool {
        if entries.len() < 3 {
            return false;
        }

        let last_three = &entries[entries.len() - 3..];

        // Rate threshold: all >= 99%
        let rate_met = last_three.iter().all(|e| e.rate >= 0.99);

        // Stability: all deltas < 0.5%
        let stable = last_three.iter().all(|e| e.delta.abs() < 0.005);

        rate_met && stable
    }
}
















include!("runner_cont_4.rs");
