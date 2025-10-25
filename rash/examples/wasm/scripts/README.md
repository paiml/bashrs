# bashrs WASM Runtime - Example Scripts

Collection of example bash scripts demonstrating the bashrs WASM runtime capabilities.

## Examples

### 01-hello-world.sh
**Difficulty**: Beginner
**Concepts**: Basic echo command

The simplest possible script - prints a greeting message.

```bash
echo "Hello, World!"
```

**Expected Output:**
```
Hello, World!
Welcome to bashrs WASM runtime
```

---

### 02-variables.sh
**Difficulty**: Beginner
**Concepts**: Variable assignment, variable expansion

Demonstrates how to assign and use variables.

```bash
name="Claude"
echo "My name is $name"
```

**Expected Output:**
```
My name is Claude
Hello, World!
Version: 1.0
WASM is awesome
WASM is awesome
```

---

### 03-navigation.sh
**Difficulty**: Beginner
**Concepts**: cd, pwd, virtual filesystem

Shows how directory navigation works in the virtual filesystem.

```bash
pwd
cd /tmp
pwd
```

**Expected Output:**
```
=== Directory Navigation Demo ===
Starting location:
/

Moving to /tmp:
/tmp

Moving to /home:
/home

Back to root:
/

Navigation complete!
```

---

### 04-deployment.sh
**Difficulty**: Intermediate
**Concepts**: Variables, navigation, realistic workflow

A realistic deployment script using variables and directory navigation.

```bash
APP_NAME="myapp"
VERSION="2.1.0"
echo "Deploying $APP_NAME v$VERSION"
```

**Expected Output:**
```
=== Deployment Script ===

Application: myapp
Version: 2.1.0
Environment: production

Creating release: release-2.1.0
...
=== Deployment Complete ===
```

---

### 05-complex-workflow.sh
**Difficulty**: Advanced
**Concepts**: Multi-step workflows, many variables, complex logic

A complex multi-step workflow demonstrating advanced usage.

```bash
PROJECT="webapp"
STAGE="development"
# ... multiple steps
```

**Expected Output:**
```
=== Complex Workflow Demo ===

Step 1: Initialize
  Project: webapp
...
Workflow completed successfully!
```

---

## Running Examples

### In Browser

```html
<script type="module">
    import init, { execute_script } from './pkg/bashrs.js';

    await init();

    // Fetch and execute example
    const response = await fetch('./scripts/01-hello-world.sh');
    const script = await response.text();
    const result = execute_script(script);

    console.log(result.stdout);
</script>
```

### In Node.js

```javascript
import { readFileSync } from 'fs';
import init, { execute_script } from './pkg/bashrs.js';

await init();

const script = readFileSync('./scripts/01-hello-world.sh', 'utf-8');
const result = execute_script(script);

console.log(result.stdout);
```

### Using the Demo

1. Open `http://localhost:8001/runtime-demo.html`
2. Load an example using the example buttons
3. Or paste script content into the editor
4. Click "Execute Script"

---

## Testing Examples

All examples are tested to ensure they work correctly:

```bash
# Run example tests
cd rash
cargo test --features wasm wasm::executor::tests
```

---

## Creating Your Own

### Template

```bash
#!/bin/sh
# Example N: [Title]
# [Description]

# Your script here
echo "Hello from my script!"
```

### Best Practices

1. **Add comments** - Explain what your script does
2. **Use variables** - Make scripts configurable
3. **Echo progress** - Show what's happening
4. **Keep it simple** - Remember current limitations

### Current Limitations

See [RUNTIME-USAGE.md](../RUNTIME-USAGE.md#limitations) for full list.

**Not supported yet:**
- Pipes (`|`)
- Redirects (`>`, `<`)
- Loops (`for`, `while`)
- Conditionals (`if`, `case`)
- Functions
- External commands (`ls`, `grep`, etc.)

---

## Contributing Examples

Have a great example? Contributions welcome!

1. Create your example script
2. Test it with the runtime demo
3. Add documentation here
4. Submit a pull request

---

**🤖 Generated with [Claude Code](https://claude.com/claude-code)**
**Co-Authored-By: Claude <noreply@anthropic.com>**
