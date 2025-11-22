# bashrs REPL Examples

This directory contains 11 comprehensive real-world examples demonstrating how to use the bashrs REPL effectively for shell script development, security auditing, and DevOps workflows.

## 📚 Examples Overview

| Example | Topic | Difficulty | Time | Key Concepts |
|---------|-------|------------|------|--------------|
| [01_basic_workflow.sh](./01_basic_workflow.sh) | Basic REPL Workflow | Beginner | 10 min | Commands, modes, parsing, linting |
| [02_security_audit.sh](./02_security_audit.sh) | Security Auditing | Intermediate | 15 min | Linting, security rules, vulnerabilities |
| [03_purification_workflow.sh](./03_purification_workflow.sh) | Purification & Idempotency | Intermediate | 20 min | Idempotency, determinism, transformations |
| [04_explain_mode.sh](./04_explain_mode.sh) | Learning Bash Constructs | Beginner | 15 min | Explain mode, parameter expansions, control flow |
| [05_script_loading.sh](./05_script_loading.sh) | Script Analysis | Intermediate | 15 min | Loading scripts, function extraction, reloading |
| [06_cicd_pipeline.sh](./06_cicd_pipeline.sh) | CI/CD Development | Advanced | 25 min | Docker, Kubernetes, deployment pipelines |
| [07_configuration_management.sh](./07_configuration_management.sh) | Config File Management | Intermediate | 20 min | .bashrc/.zshrc, PATH, aliases, functions |
| [08_multiline_editing.sh](./08_multiline_editing.sh) | Multi-line Input | Intermediate | 15 min | Functions, loops, conditionals, heredocs |
| [09_tab_completion.sh](./09_tab_completion.sh) | Tab Completion | Beginner | 10 min | Completion, keyboard shortcuts, efficiency |
| [10_variables_session.sh](./10_variables_session.sh) | Variable Management | Intermediate | 15 min | Variables, session state, environments |
| [11_troubleshooting.sh](./11_troubleshooting.sh) | Debugging & Troubleshooting | Advanced | 20 min | Common issues, solutions, recovery |

**Total**: 11 examples covering beginner to advanced topics

## 🚀 Quick Start

### Prerequisites

```bash
# Install bashrs
cargo install bashrs

# Verify installation
bashrs --version
```

### Running Examples

These are interactive examples meant to be followed along in the REPL:

```bash
# Read an example (they're shell scripts with embedded instructions)
cat rash/examples/repl/01_basic_workflow.sh

# Start the REPL
bashrs repl

# Follow along with the example instructions
# Each example shows commands to type and expected output
```

## 📖 Learning Path

### For Beginners

Start with these examples if you're new to bashrs or REPL:

1. **01_basic_workflow.sh** - Learn fundamental REPL commands
2. **04_explain_mode.sh** - Understand bash constructs interactively
3. **09_tab_completion.sh** - Speed up your workflow with completion

### For Intermediate Users

Progress to these for practical workflows:

4. **02_security_audit.sh** - Audit scripts for security issues
5. **03_purification_workflow.sh** - Make scripts idempotent and safe
6. **05_script_loading.sh** - Analyze and iterate on complete scripts
7. **07_configuration_management.sh** - Clean up shell configs
8. **08_multiline_editing.sh** - Master complex interactive editing
9. **10_variables_session.sh** - Manage session state effectively

### For Advanced Users

Master these for production workflows:

10. **06_cicd_pipeline.sh** - Build CI/CD deployment pipelines
11. **11_troubleshooting.sh** - Debug and resolve common issues

## 🎯 Use Cases

### Security Auditing

Use the REPL to find and fix security vulnerabilities:

```bash
# See: 02_security_audit.sh
bashrs repl
bashrs [normal]> :mode lint
bashrs [lint]> eval $USER_INPUT  # Dangerous!
Found 1 issue: SEC001 - Command injection risk
```

### CI/CD Development

Build deployment pipelines interactively:

```bash
# See: 06_cicd_pipeline.sh
bashrs repl
bashrs [normal]> app=myapp
bashrs [normal]> version=v2.0
bashrs [normal]> :mode purify
bashrs [purify]> docker build -t $app:$version .
✓ Purified: docker build -t "$app:$version" .
```

### Configuration Management

Clean up messy shell configs:

```bash
# See: 07_configuration_management.sh
bashrs repl
bashrs [normal]> :load ~/.bashrc
bashrs [normal]> :mode lint
bashrs [lint]> # View issues...
bashrs [lint]> :mode purify
bashrs [purify]> # Get fixed versions...
```

### Learning Bash

Understand bash constructs interactively:

```bash
# See: 04_explain_mode.sh
bashrs repl
bashrs [normal]> :mode explain
bashrs [explain]> ${var:-default}
📖 Parameter Expansion: Use Default Value...
```

## 💡 Tips for Using Examples

1. **Read First, Then Try**
   - Read the entire example first
   - Understand the workflow
   - Then follow along in the REPL

2. **Experiment Freely**
   - Modify commands to see what happens
   - Try variations
   - Use :history to track what you tried

3. **Combine Techniques**
   - Mix concepts from different examples
   - Build on what you learned
   - Create your own workflows

4. **Practice Regularly**
   - Repetition builds muscle memory
   - Try examples multiple times
   - Apply to your own scripts

5. **Reference Back**
   - Keep examples handy
   - Use as templates for your workflows
   - Share with your team

## 📊 Example Statistics

- **Total Examples**: 11
- **Lines of Documentation**: ~3,500
- **Topics Covered**: 50+
- **Use Cases**: 30+
- **Commands Demonstrated**: 100+

## 🔗 Related Documentation

- **[Tutorial: Your First REPL Session](../../book/src/repl/tutorial.md)** - Step-by-step beginner tutorial
- **[REPL User Guide](../../book/src/repl/user-guide.md)** - Comprehensive feature reference
- **[REPL Commands Reference](../../book/src/reference/repl-commands.md)** - Complete command documentation
- **[Interactive REPL (Getting Started)](../../book/src/getting-started/repl.md)** - Quick start guide

## 🛠 Development

These examples are tested and maintained alongside bashrs:

```bash
# Run example validation (if implemented)
cargo test --test example_validation

# Update examples
edit rash/examples/repl/XX_topic.sh

# Add new example
cp rash/examples/repl/01_basic_workflow.sh rash/examples/repl/12_new_topic.sh
# Update content and README.md
```

## 📝 Example Format

Each example follows this structure:

```bash
#!/bin/bash
# REPL Example XX: Title
# Brief description
#
# This example shows:
# - Feature 1
# - Feature 2
# - Feature 3
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example XX: Title
=================================================================

Introduction and context...

STEP 1: Topic
-------------
bashrs [mode]> command
Output and explanation...

STEP 2: Topic
-------------
...

=================================================================
Key Takeaways:
=================================================================

1. Important point 1
2. Important point 2
...

Next Steps:
-----------
Try example YY_next_topic.sh ...
EOF
```

## 🤝 Contributing

Want to add an example?

1. Follow the format above
2. Focus on a specific use case or workflow
3. Include detailed explanations
4. Add expected output
5. Test in the REPL
6. Update this README.md
7. Submit a PR

## 📧 Feedback

Found an issue with an example? Have a suggestion?

- **File an issue**: https://github.com/paiml/bashrs/issues
- **Start a discussion**: https://github.com/paiml/bashrs/discussions
- **Submit a PR**: https://github.com/paiml/bashrs/pulls

## 📄 License

These examples are part of the bashrs project and are distributed under the same license.

---

**Happy scripting!** 🚀

These examples are maintained by the bashrs project.
Last updated: 2024-11-22
