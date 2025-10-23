/**
 * bashrs WASM API Wrapper for WOS (Web Operating System)
 *
 * Provides a clean, Promise-based API for WOS integration.
 *
 * @version 6.2.0
 * @license MIT
 */

import initWasmModule, {
  version,
  analyze_config,
  purify_config
} from '../pkg/bashrs.js';

class BashrsWOS {
  constructor() {
    this.ready = false;
    this.initPromise = null;
  }

  /**
   * Initialize bashrs WASM module
   * @returns {Promise<void>}
   */
  async init() {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = (async () => {
      try {
        await initWasmModule();
        this.ready = true;
        console.log(`[bashrs-wos] Initialized v${version()}`);
      } catch (err) {
        console.error('[bashrs-wos] Initialization failed:', err);
        throw new Error(`Failed to initialize bashrs WASM: ${err.message}`);
      }
    })();

    return this.initPromise;
  }

  /**
   * Get bashrs version
   * @returns {string} Version string (e.g., "6.2.0")
   */
  getVersion() {
    this._ensureReady();
    return version();
  }

  /**
   * Analyze shell configuration
   *
   * @param {string} content - Shell config content (.bashrc, .zshrc, etc.)
   * @param {string} [filename=".bashrc"] - Optional filename for context
   * @returns {Promise<AnalysisResult>} Analysis results
   *
   * @typedef {Object} AnalysisResult
   * @property {number} issue_count - Number of issues found
   * @property {number} line_count - Lines in the config
   * @property {number} complexity_score - Complexity score (0-10)
   * @property {Array<Issue>} issues - List of detected issues
   *
   * @typedef {Object} Issue
   * @property {string} rule_id - Rule identifier (e.g., "CONFIG-001")
   * @property {string} severity - Severity level
   * @property {number} line - Line number
   * @property {number} column - Column number
   * @property {string} message - Issue description
   * @property {string|null} suggestion - Optional fix suggestion
   *
   * @example
   * const result = await bashrs.analyzeConfig(`
   *   export PATH="/usr/local/bin:$PATH"
   *   export PATH="/usr/local/bin:$PATH"  # Duplicate!
   * `);
   *
   * console.log(`Found ${result.issue_count} issues`);
   * // Output: Found 1 issues
   */
  async analyzeConfig(content, filename = '.bashrc') {
    this._ensureReady();

    try {
      const result = analyze_config(content, filename);

      return {
        issue_count: result.issue_count,
        line_count: result.line_count,
        complexity_score: result.complexity_score,
        issues: JSON.parse(result.issues_json)
      };
    } catch (err) {
      throw new Error(`Config analysis failed: ${err.message}`);
    }
  }

  /**
   * Purify shell configuration
   *
   * Transforms config to be deterministic and idempotent.
   *
   * @param {string} content - Shell config content
   * @returns {Promise<string>} Purified config
   *
   * @example
   * const purified = await bashrs.purifyConfig(`
   *   mkdir /tmp/test
   *   rm /tmp/test/file
   * `);
   *
   * console.log(purified);
   * // Output:
   * // mkdir -p /tmp/test
   * // rm -f /tmp/test/file
   */
  async purifyConfig(content) {
    this._ensureReady();

    try {
      return purify_config(content);
    } catch (err) {
      throw new Error(`Config purification failed: ${err.message}`);
    }
  }

  /**
   * Lint shell script (convenience method)
   *
   * @param {string} content - Shell script content
   * @param {string} [filename="script.sh"] - Optional filename
   * @returns {Promise<AnalysisResult>} Linting results
   */
  async lintScript(content, filename = 'script.sh') {
    return this.analyzeConfig(content, filename);
  }

  /**
   * Check if module is ready
   * @returns {boolean} True if initialized
   */
  isReady() {
    return this.ready;
  }

  /**
   * Internal: Ensure module is initialized
   * @private
   */
  _ensureReady() {
    if (!this.ready) {
      throw new Error('bashrs WASM not initialized. Call init() first.');
    }
  }
}

// Export singleton instance for WOS
const bashrsWOS = new BashrsWOS();

export default bashrsWOS;
export { BashrsWOS };
