use crate::models::{Config, Error, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Adaptive debouncer for transpilation requests
pub struct AdaptiveDebouncer {
    base_delay_ms: u64,
    burst_threshold: usize,
    history: VecDeque<Instant>,
}

impl Default for AdaptiveDebouncer {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveDebouncer {
    pub fn new() -> Self {
        Self {
            base_delay_ms: 150,
            burst_threshold: 5,
            history: VecDeque::new(),
        }
    }

    pub fn calculate_delay(&mut self) -> Duration {
        let now = Instant::now();

        // Remove old entries (older than 1 second)
        while let Some(front) = self.history.front() {
            if now.duration_since(*front) > Duration::from_secs(1) {
                self.history.pop_front();
            } else {
                break;
            }
        }

        // Adaptive delay based on burst detection
        let delay = if self.history.len() > self.burst_threshold {
            // User is typing rapidly, increase delay
            Duration::from_millis(self.base_delay_ms * 2)
        } else {
            Duration::from_millis(self.base_delay_ms)
        };

        self.history.push_back(now);
        delay
    }
}

/// Structured concurrency token for cancellation
#[derive(Clone)]
pub struct CancellationToken {
    #[cfg(feature = "playground")]
    inner: Arc<std::sync::atomic::AtomicBool>,

    #[cfg(not(feature = "playground"))]
    _placeholder: std::marker::PhantomData<()>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn new() -> Self {
        #[cfg(feature = "playground")]
        {
            Self {
                inner: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }

        #[cfg(not(feature = "playground"))]
        {
            Self {
                _placeholder: std::marker::PhantomData,
            }
        }
    }

    pub fn child_token(&self) -> Self {
        self.clone()
    }

    pub fn cancel(&self) {
        #[cfg(feature = "playground")]
        {
            self.inner.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    pub fn is_cancelled(&self) -> bool {
        #[cfg(feature = "playground")]
        {
            self.inner.load(std::sync::atomic::Ordering::SeqCst)
        }

        #[cfg(not(feature = "playground"))]
        {
            false
        }
    }

    pub fn check_cancelled(&self) -> Result<()> {
        if self.is_cancelled() {
            Err(Error::Internal("Operation cancelled".to_string()))
        } else {
            Ok(())
        }
    }
}

/// Transpilation output with diagnostics and metrics
#[derive(Debug, Clone)]
pub struct TranspilationOutput {
    pub shell_code: String,
    pub diagnostics: Vec<String>,
    pub metrics: TranspilationMetrics,
}

/// Performance metrics for transpilation
#[derive(Debug, Clone)]
pub struct TranspilationMetrics {
    pub parse_time: Duration,
    pub validate_time: Duration,
    pub transpile_time: Duration,
    pub total_time: Duration,
    pub ast_node_count: usize,
    pub shell_line_count: usize,
}

impl Default for TranspilationMetrics {
    fn default() -> Self {
        Self {
            parse_time: Duration::ZERO,
            validate_time: Duration::ZERO,
            transpile_time: Duration::ZERO,
            total_time: Duration::ZERO,
            ast_node_count: 0,
            shell_line_count: 0,
        }
    }
}

/// Cancellable transpilation controller for playground
pub struct TranspilationController {
    #[cfg(feature = "playground")]
    current_generation: std::sync::atomic::AtomicU64,

    cancel_token: CancellationToken,
    debouncer: AdaptiveDebouncer,
    config: Config,
}

impl TranspilationController {
    pub fn new(config: Config) -> Self {
        Self {
            #[cfg(feature = "playground")]
            current_generation: std::sync::atomic::AtomicU64::new(0),

            cancel_token: CancellationToken::new(),
            debouncer: AdaptiveDebouncer::new(),
            config,
        }
    }

    /// Transpile source with cancellation support
    pub async fn transpile_with_cancellation(
        &mut self,
        source: Arc<str>,
        generation: u64,
    ) -> Result<TranspilationOutput> {
        #[cfg(feature = "playground")]
        {
            use std::sync::atomic::Ordering;

            let token = CancellationToken::new();
            let child_token = token.child_token();

            // Update generation and cancel previous
            let prev_gen = self.current_generation.swap(generation, Ordering::SeqCst);
            if prev_gen < generation {
                self.cancel_token.cancel();
                self.cancel_token = token;
            }

            // Add debounce delay
            let delay = self.debouncer.calculate_delay();
            tokio::time::sleep(delay).await;

            // Check if still latest generation
            if self.current_generation.load(Ordering::SeqCst) != generation {
                return Err(Error::Internal("Superseded by newer request".to_string()));
            }

            // Transpile with cancellation checks
            let child_token_clone = child_token.clone();
            tokio::select! {
                result = self.transpile_internal(source, child_token) => {
                    result
                }
                _ = async {
                    while !child_token_clone.is_cancelled() {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                } => {
                    Err(Error::Internal("Cancelled".to_string()))
                }
            }
        }

        #[cfg(not(feature = "playground"))]
        {
            self.transpile_sync(&source)
        }
    }

    async fn transpile_internal(
        &self,
        source: Arc<str>,
        cancel: CancellationToken,
    ) -> Result<TranspilationOutput> {
        let total_start = Instant::now();
        let mut metrics = TranspilationMetrics::default();

        // Parse phase
        cancel.check_cancelled()?;
        let parse_start = Instant::now();
        let ast = crate::services::parser::parse(&source)?;
        metrics.parse_time = parse_start.elapsed();
        metrics.ast_node_count = self.count_ast_nodes(&ast);

        // Validation phase with cancellation points
        cancel.check_cancelled()?;
        let validate_start = Instant::now();
        crate::ast::validate(&ast)?;

        // TODO: Add more granular validation with cancellation points
        let diagnostics = Vec::new();

        metrics.validate_time = validate_start.elapsed();

        // Transpilation phase
        cancel.check_cancelled()?;
        let transpile_start = Instant::now();

        let ir = crate::ir::from_ast(&ast)?;
        cancel.check_cancelled()?;

        let optimized = crate::ir::optimize(ir, &self.config)?;
        cancel.check_cancelled()?;

        let shell_code = crate::emitter::emit(&optimized, &self.config)?;
        metrics.transpile_time = transpile_start.elapsed();
        metrics.shell_line_count = shell_code.lines().count();

        metrics.total_time = total_start.elapsed();

        Ok(TranspilationOutput {
            shell_code,
            diagnostics,
            metrics,
        })
    }

    #[allow(dead_code)]
    fn transpile_sync(&self, source: &str) -> Result<TranspilationOutput> {
        let total_start = Instant::now();
        let mut metrics = TranspilationMetrics::default();

        // Parse
        let parse_start = Instant::now();
        let ast = crate::services::parser::parse(source)?;
        metrics.parse_time = parse_start.elapsed();
        metrics.ast_node_count = self.count_ast_nodes(&ast);

        // Validate
        let validate_start = Instant::now();
        crate::ast::validate(&ast)?;
        metrics.validate_time = validate_start.elapsed();

        // Transpile
        let transpile_start = Instant::now();
        let ir = crate::ir::from_ast(&ast)?;
        let optimized = crate::ir::optimize(ir, &self.config)?;
        let shell_code = crate::emitter::emit(&optimized, &self.config)?;
        metrics.transpile_time = transpile_start.elapsed();
        metrics.shell_line_count = shell_code.lines().count();

        metrics.total_time = total_start.elapsed();

        Ok(TranspilationOutput {
            shell_code,
            diagnostics: Vec::new(),
            metrics,
        })
    }

    fn count_ast_nodes(&self, _ast: &crate::ast::restricted::RestrictedAst) -> usize {
        // TODO: Implement proper AST node counting
        1
    }
}
