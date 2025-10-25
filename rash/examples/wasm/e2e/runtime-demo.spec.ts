/**
 * Runtime Demo E2E Tests
 *
 * Tests the bash runtime execution in the browser.
 */

import { test, expect } from '@playwright/test';

test.describe('WASM Bash Runtime Demo', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:8001/runtime-demo.html');

    // Wait for WASM to load
    await expect(page.locator('#loading-status')).toContainText('WASM loaded successfully', { timeout: 10000 });
  });

  test('R01: Page loads successfully', async ({ page }) => {
    // ARRANGE & ACT: Page already loaded in beforeEach

    // ASSERT
    await expect(page.locator('h1')).toContainText('bashrs WASM Runtime Demo');
    await expect(page.locator('#script-input')).toBeVisible();
    await expect(page.locator('#output')).toBeVisible();
  });

  test('R02: Execute simple echo command', async ({ page }) => {
    // ARRANGE
    await page.locator('#script-input').fill('echo "Hello, WASM!"');

    // ACT
    await page.locator('button:has-text("Execute Script")').click();

    // ASSERT
    await expect(page.locator('#output')).toContainText('Hello, WASM!');
    await expect(page.locator('#exit-code')).toHaveText('0');
  });

  test('R03: Execute variable assignment and expansion', async ({ page }) => {
    // ARRANGE
    const script = `name="Claude"
greeting="Hello"
echo "$greeting, $name!"`;
    await page.locator('#script-input').fill(script);

    // ACT
    await page.locator('button:has-text("Execute Script")').click();

    // ASSERT
    await expect(page.locator('#output')).toContainText('Hello, Claude!');
    await expect(page.locator('#exit-code')).toHaveText('0');
  });

  test('R04: Execute cd and pwd commands', async ({ page }) => {
    // ARRANGE
    const script = `cd /tmp
pwd`;
    await page.locator('#script-input').fill(script);

    // ACT
    await page.locator('button:has-text("Execute Script")').click();

    // ASSERT
    await expect(page.locator('#output')).toContainText('/tmp');
    await expect(page.locator('#exit-code')).toHaveText('0');
  });

  test('R05: Execute multi-line script', async ({ page }) => {
    // ARRANGE
    const script = `echo "Line 1"
echo "Line 2"
echo "Line 3"`;
    await page.locator('#script-input').fill(script);

    // ACT
    await page.locator('button:has-text("Execute Script")').click();

    // ASSERT
    const output = await page.locator('#output').textContent();
    expect(output).toContain('Line 1');
    expect(output).toContain('Line 2');
    expect(output).toContain('Line 3');
    await expect(page.locator('#exit-code')).toHaveText('0');
  });

  test('R06: Load example script', async ({ page }) => {
    // ARRANGE & ACT
    await page.locator('button:has-text("Variables")').click();

    // ASSERT
    const scriptContent = await page.locator('#script-input').inputValue();
    expect(scriptContent).toContain('name=');
    expect(scriptContent).toContain('echo');
  });

  test('R07: Clear functionality', async ({ page }) => {
    // ARRANGE
    await page.locator('#script-input').fill('echo "test"');
    await page.locator('button:has-text("Execute Script")').click();
    await expect(page.locator('#output')).toContainText('test');

    // ACT
    await page.locator('button:has-text("Clear")').click();

    // ASSERT
    await expect(page.locator('#script-input')).toHaveValue('');
    await expect(page.locator('#output')).toBeEmpty();
    await expect(page.locator('#exit-code')).toHaveText('-');
  });

  test('R08: Execution metrics display', async ({ page }) => {
    // ARRANGE
    await page.locator('#script-input').fill('echo "test"');

    // ACT
    await page.locator('button:has-text("Execute Script")').click();

    // ASSERT
    await expect(page.locator('#exit-code')).toHaveText('0');
    await expect(page.locator('#exec-time')).toContainText('ms');
    await expect(page.locator('#output-lines')).not.toHaveText('-');
  });

  test('R09: Complex script execution', async ({ page }) => {
    // ARRANGE
    const script = `# Complex example
name="WASM"
version="1.0"
echo "Welcome to $name v$version"
cd /tmp
pwd
echo "Done!"`;
    await page.locator('#script-input').fill(script);

    // ACT
    await page.locator('button:has-text("Execute Script")').click();

    // ASSERT
    const output = await page.locator('#output').textContent();
    expect(output).toContain('Welcome to WASM v1.0');
    expect(output).toContain('/tmp');
    expect(output).toContain('Done!');
    await expect(page.locator('#exit-code')).toHaveText('0');
  });

  test('R10: Error handling for unknown command', async ({ page }) => {
    // ARRANGE
    await page.locator('#script-input').fill('unknowncommand');

    // ACT
    await page.locator('button:has-text("Execute Script")').click();

    // ASSERT
    await expect(page.locator('#output')).toContainText('Error');
    await expect(page.locator('#exit-code')).toHaveText('1');
  });
});
