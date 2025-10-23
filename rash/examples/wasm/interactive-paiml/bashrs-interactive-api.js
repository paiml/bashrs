/**
 * bashrs WASM API Wrapper for interactive.paiml.com
 *
 * Educational wrapper providing real-time bash linting feedback
 * for interactive learning platform.
 *
 * @version 6.2.0
 * @license MIT
 */

import initWasmModule, {
  version,
  analyze_config,
  purify_config
} from '../pkg/bashrs.js';

class BashrsInteractive {
  constructor() {
    this.ready = false;
    this.initPromise = null;
    this.debounceTimer = null;
    this.listeners = new Map();
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
        console.log(`[bashrs-interactive] Initialized v${version()}`);
        this._emit('ready', { version: version() });
      } catch (err) {
        console.error('[bashrs-interactive] Initialization failed:', err);
        throw new Error(`Failed to initialize bashrs WASM: ${err.message}`);
      }
    })();

    return this.initPromise;
  }

  /**
   * Lint bash code with real-time feedback
   *
   * Provides educational feedback suitable for learning environments.
   * Includes issue severity, explanations, and fix suggestions.
   *
   * @param {string} content - Bash code to lint
   * @param {Object} options - Linting options
   * @param {boolean} options.educational - Include educational explanations (default: true)
   * @param {string} options.filename - Optional filename
   * @returns {Promise<LintResult>} Linting results with educational feedback
   *
   * @example
   * const result = await bashrs.lintCode(`
   *   export PATH=$HOME/bin:$PATH
   * `);
   *
   * console.log(result.issues[0].educational);
   * // Output: "Variable expansions should be quoted to handle spaces..."
   */
  async lintCode(content, options = {}) {
    this._ensureReady();

    const {
      educational = true,
      filename = 'lesson.sh'
    } = options;

    try {
      const result = analyze_config(content, filename);

      const issues = JSON.parse(result.issues_json).map(issue => {
        const enhanced = { ...issue };

        if (educational) {
          enhanced.educational = this._getEducationalExplanation(issue.rule_id);
          enhanced.learnMore = this._getLearnMoreLink(issue.rule_id);
        }

        return enhanced;
      });

      return {
        issue_count: result.issue_count,
        line_count: result.line_count,
        complexity_score: result.complexity_score,
        issues,
        educational: educational ? this._getGeneralTips(issues) : null
      };
    } catch (err) {
      throw new Error(`Linting failed: ${err.message}`);
    }
  }

  /**
   * Lint with real-time feedback (debounced)
   *
   * Optimized for "as-you-type" linting with debouncing
   * to avoid excessive analysis while typing.
   *
   * @param {string} content - Bash code to lint
   * @param {Function} callback - Callback(result)
   * @param {number} delay - Debounce delay in ms (default: 300)
   *
   * @example
   * editor.on('change', (code) => {
   *   bashrs.lintRealtime(code, (result) => {
   *     updateIssuesPanel(result.issues);
   *   });
   * });
   */
  lintRealtime(content, callback, delay = 300) {
    this._ensureReady();

    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    this.debounceTimer = setTimeout(async () => {
      try {
        const result = await this.lintCode(content);
        callback(result);
        this._emit('lint', result);
      } catch (err) {
        callback({ error: err.message });
        this._emit('error', err);
      }
    }, delay);
  }

  /**
   * Get example lesson
   *
   * Returns pre-configured lesson with intentional issues
   * for educational purposes.
   *
   * @param {string} lessonId - Lesson identifier
   * @returns {Object} Lesson with code, description, and learning objectives
   *
   * @example
   * const lesson = bashrs.getLesson('config-001-path');
   * console.log(lesson.code);      // Code with intentional issue
   * console.log(lesson.objective);  // What to learn
   */
  getLesson(lessonId) {
    const lessons = {
      'config-001-path': {
        id: 'config-001-path',
        title: 'PATH Deduplication',
        objective: 'Learn to avoid duplicate PATH entries',
        code: `# Adding directories to PATH
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # What's wrong here?`,
        expectedIssues: ['CONFIG-001'],
        hint: 'Look for repeated directory entries in PATH',
        solution: 'Remove the duplicate /usr/local/bin entry on line 3'
      },
      'config-002-quotes': {
        id: 'config-002-quotes',
        title: 'Variable Quoting',
        objective: 'Learn when and why to quote variable expansions',
        code: `# Setting up project directory
export PROJECT_DIR=$HOME/my projects
export EDITOR=vim`,
        expectedIssues: ['CONFIG-002'],
        hint: 'What happens if $HOME contains spaces?',
        solution: 'Quote the variable expansion: "$HOME/my projects"'
      },
      'config-003-aliases': {
        id: 'config-003-aliases',
        title: 'Alias Management',
        objective: 'Learn to avoid conflicting alias definitions',
        code: `# Shell aliases
alias ls='ls --color=auto'
alias ll='ls -lah'
alias ls='ls -G'  # Redefining ls - is this intentional?`,
        expectedIssues: ['CONFIG-003'],
        hint: 'The same alias is defined twice with different values',
        solution: 'Remove one of the ls alias definitions'
      },
      'config-004-determinism': {
        id: 'config-004-determinism',
        title: 'Deterministic Configs',
        objective: 'Learn why $RANDOM and timestamps cause problems',
        code: `# Session setup
export SESSION_ID=$RANDOM
export BUILD_TAG="build-$(date +%s)"
export USER_NAME="alice"`,
        expectedIssues: ['CONFIG-004', 'CONFIG-004'],
        hint: 'Which values will be different every time?',
        solution: 'Replace $RANDOM and $(date) with deterministic values'
      }
    };

    return lessons[lessonId] || null;
  }

  /**
   * Get all available lessons
   * @returns {Array<Object>} List of lesson metadata
   */
  getAllLessons() {
    return [
      { id: 'config-001-path', title: 'PATH Deduplication', difficulty: 'beginner' },
      { id: 'config-002-quotes', title: 'Variable Quoting', difficulty: 'beginner' },
      { id: 'config-003-aliases', title: 'Alias Management', difficulty: 'beginner' },
      { id: 'config-004-determinism', title: 'Deterministic Configs', difficulty: 'intermediate' }
    ];
  }

  /**
   * Check lesson solution
   *
   * Validates if the student's code fixes the lesson's issues.
   *
   * @param {string} lessonId - Lesson identifier
   * @param {string} studentCode - Student's solution
   * @returns {Promise<Object>} Validation result
   */
  async checkLessonSolution(lessonId, studentCode) {
    const lesson = this.getLesson(lessonId);
    if (!lesson) {
      throw new Error(`Unknown lesson: ${lessonId}`);
    }

    const result = await this.lintCode(studentCode, { educational: false });
    const foundIssues = result.issues.map(i => i.rule_id);
    const expectedIssues = lesson.expectedIssues;

    const fixed = expectedIssues.filter(rule => !foundIssues.includes(rule));
    const remaining = expectedIssues.filter(rule => foundIssues.includes(rule));

    return {
      success: remaining.length === 0,
      fixed,
      remaining,
      message: remaining.length === 0
        ? '✅ Perfect! All issues fixed!'
        : `🔧 Still need to fix: ${remaining.join(', ')}`
    };
  }

  /**
   * Register event listener
   * @param {string} event - Event name ('ready', 'lint', 'error')
   * @param {Function} callback - Callback function
   */
  on(event, callback) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event).push(callback);
  }

  /**
   * Get bashrs version
   * @returns {string} Version string
   */
  getVersion() {
    this._ensureReady();
    return version();
  }

  /**
   * Check if ready
   * @returns {boolean} True if initialized
   */
  isReady() {
    return this.ready;
  }

  // Internal methods

  _ensureReady() {
    if (!this.ready) {
      throw new Error('bashrs WASM not initialized. Call init() first.');
    }
  }

  _emit(event, data) {
    if (this.listeners.has(event)) {
      this.listeners.get(event).forEach(callback => callback(data));
    }
  }

  _getEducationalExplanation(ruleId) {
    const explanations = {
      'CONFIG-001': 'Duplicate PATH entries waste memory and slow down command lookups. Keep PATH clean by adding each directory only once.',
      'CONFIG-002': 'Unquoted variable expansions can break if the value contains spaces or special characters. Always quote variables: "$VAR" instead of $VAR.',
      'CONFIG-003': 'Redefining the same alias multiple times creates confusion. Each alias should have one clear definition.',
      'CONFIG-004': 'Using $RANDOM, timestamps, or process IDs makes your config non-deterministic. This breaks reproducibility and testing.'
    };
    return explanations[ruleId] || 'See documentation for more information.';
  }

  _getLearnMoreLink(ruleId) {
    return `https://interactive.paiml.com/docs/bash-linting/${ruleId.toLowerCase()}`;
  }

  _getGeneralTips(issues) {
    if (issues.length === 0) {
      return {
        message: '✅ Great job! Your bash code follows best practices.',
        tips: [
          'Keep your code deterministic and reproducible',
          'Always quote variable expansions',
          'Avoid duplicate definitions'
        ]
      };
    }

    return {
      message: `Found ${issues.length} issue(s) to fix`,
      tips: [
        'Read the educational explanation for each issue',
        'Try fixing one issue at a time',
        'Test your changes after each fix'
      ]
    };
  }
}

// Export singleton instance
const bashrsInteractive = new BashrsInteractive();

export default bashrsInteractive;
export { BashrsInteractive };
