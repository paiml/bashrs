impl Default for InstallerGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate step duration based on action type
fn estimate_step_duration(step: &super::spec::Step) -> f64 {
    // Parse timeout if specified
    if let Some(ref timeout) = step.timing.timeout {
        if let Some(secs) = parse_duration_secs(timeout) {
            // Estimate as 10% of timeout (conservative)
            return secs * 0.1;
        }
    }

    // Default estimates based on action type
    match step.action.as_str() {
        "verify" => 0.5,
        "script" => 5.0,
        "apt-install" => 30.0,
        "apt-remove" => 10.0,
        "file-write" => 0.5,
        "user-group" => 1.0,
        "user-add-to-group" => 1.0,
        _ => 5.0,
    }
}

/// Parse duration string (e.g., "5m", "30s") to seconds
fn parse_duration_secs(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(num) = s.strip_suffix('s') {
        num.parse().ok()
    } else if let Some(num) = s.strip_suffix('m') {
        num.parse::<f64>().ok().map(|m| m * 60.0)
    } else if let Some(num) = s.strip_suffix('h') {
        num.parse::<f64>().ok().map(|h| h * 3600.0)
    } else {
        s.parse().ok()
    }
}

/// Extract required capabilities from step
fn extract_capabilities(step: &super::spec::Step) -> Vec<String> {
    let mut caps = Vec::new();

    match step.action.as_str() {
        "apt-install" | "apt-remove" => caps.push("apt".to_string()),
        "script" => {
            if let Some(ref script) = step.script {
                if script.content.contains("docker") {
                    caps.push("docker".to_string());
                }
            }
        }
        _ => {}
    }

    caps
}

/// sccache client for build artifact caching
#[derive(Debug, Clone)]
pub struct SccacheClient {
    /// Server address
    server: String,
    /// Whether connected
    connected: bool,
}

impl SccacheClient {
    /// Create new sccache client
    pub fn new(server: &str) -> Self {
        Self {
            server: server.to_string(),
            connected: false,
        }
    }

    /// Connect to sccache server
    pub fn connect(&mut self) -> Result<()> {
        // In a real implementation, this would establish connection
        // For now, just validate the address format
        if self.server.contains(':') {
            self.connected = true;
            Ok(())
        } else {
            Err(Error::Validation(format!(
                "Invalid sccache server address: {}",
                self.server
            )))
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get server address
    pub fn server(&self) -> &str {
        &self.server
    }

    /// Get cache stats (placeholder)
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: 0,
            misses: 0,
            size_bytes: 0,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache size in bytes
    pub size_bytes: u64,
}

impl CacheStats {
    /// Hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            (self.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Format execution plan for display
pub fn format_execution_plan(plan: &ExecutionPlan, max_parallel: usize) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "Execution Plan ({} waves, max parallelism: {})\n",
        plan.waves.len(),
        max_parallel
    ));
    output.push_str(
        "══════════════════════════════════════════════════════════════════════════════\n\n",
    );

    for wave in &plan.waves {
        let wave_type = if wave.is_sequential {
            format!(
                "sequential - {}",
                wave.sequential_reason.as_deref().unwrap_or("constraint")
            )
        } else {
            "parallel".to_string()
        };

        output.push_str(&format!("Wave {} ({}):\n", wave.wave_number + 1, wave_type));

        for step_id in &wave.step_ids {
            output.push_str(&format!("  • {}\n", step_id));
        }

        output.push_str(&format!(
            "  Estimated: {:.1}s\n\n",
            wave.estimated_duration_secs
        ));
    }

    output.push_str(&format!(
        "Estimated total: {:.1}s (vs {:.1}s sequential = {:.0}% speedup)\n",
        plan.total_duration_parallel_secs,
        plan.total_duration_sequential_secs,
        plan.speedup_percent
    ));
    output.push_str(
        "══════════════════════════════════════════════════════════════════════════════\n",
    );

    output
}

#[cfg(test)]
#[path = "distributed_tests_dist_001.rs"]
mod tests_extracted;
