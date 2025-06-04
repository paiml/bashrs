#!/usr/bin/env python3
"""
Focused analysis on specific modules
"""

import json

# Load the full analysis
with open('paiml-analysis.json', 'r') as f:
    data = json.load(f)

# Extract metrics for specific modules
target_modules = ['parser', 'emitter', 'verifier', 'ir']
module_metrics = {}

for module in target_modules:
    module_files = []
    module_issues = []
    
    for file_metric in data['detailed_metrics']:
        if f'/src/{module}/' in file_metric['path']:
            module_files.append(file_metric)
    
    for issue in data['tdg_issues']:
        if f'/src/{module}/' in issue['location']:
            module_issues.append(issue)
    
    # Calculate module summary
    if module_files:
        total_complexity = sum(f['cyclomatic_complexity'] for f in module_files)
        total_lines = sum(f['lines'] for f in module_files)
        total_functions = sum(f['functions'] for f in module_files)
        
        module_metrics[module] = {
            'files': len(module_files),
            'total_lines': total_lines,
            'total_functions': total_functions,
            'total_complexity': total_complexity,
            'avg_complexity_per_file': total_complexity / len(module_files),
            'avg_complexity_per_function': total_complexity / total_functions if total_functions > 0 else 0,
            'tdg_issues': len(module_issues),
            'high_severity_issues': sum(1 for i in module_issues if i['severity'] == 'high'),
            'hotspot_files': []
        }
        
        # Find hotspot files in this module
        for hotspot in data['hotspots']:
            if f'/src/{module}/' in hotspot['file']:
                module_metrics[module]['hotspot_files'].append({
                    'file': hotspot['file'].split('/')[-1],
                    'score': hotspot['complexity_score'],
                    'cyclomatic': hotspot['cyclomatic']
                })

# Print focused analysis
print("\nFOCUSED MODULE ANALYSIS")
print("=" * 60)

for module, metrics in module_metrics.items():
    print(f"\n{module.upper()} MODULE:")
    print(f"  Files: {metrics['files']}")
    print(f"  Total Lines: {metrics['total_lines']}")
    print(f"  Total Functions: {metrics['total_functions']}")
    print(f"  Total Complexity: {metrics['total_complexity']}")
    print(f"  Avg Complexity/File: {metrics['avg_complexity_per_file']:.2f}")
    print(f"  Avg Complexity/Function: {metrics['avg_complexity_per_function']:.2f}")
    print(f"  TDG Issues: {metrics['tdg_issues']} (High: {metrics['high_severity_issues']})")
    
    if metrics['hotspot_files']:
        print(f"  Hotspot Files:")
        for hf in metrics['hotspot_files']:
            print(f"    - {hf['file']} (score: {hf['score']}, cyclomatic: {hf['cyclomatic']})")

# Summary recommendations
print("\n\nRECOMMENDATIONS:")
print("-" * 60)

high_complexity_modules = [(m, d['avg_complexity_per_function']) for m, d in module_metrics.items() if d['avg_complexity_per_function'] > 5]
if high_complexity_modules:
    print("\n1. HIGH COMPLEXITY MODULES (need refactoring):")
    for module, complexity in sorted(high_complexity_modules, key=lambda x: x[1], reverse=True):
        print(f"   - {module}: {complexity:.2f} avg complexity per function")

modules_with_tdg = [(m, d['tdg_issues']) for m, d in module_metrics.items() if d['tdg_issues'] > 0]
if modules_with_tdg:
    print("\n2. MODULES WITH TDG ISSUES:")
    for module, issues in sorted(modules_with_tdg, key=lambda x: x[1], reverse=True):
        high_issues = module_metrics[module]['high_severity_issues']
        print(f"   - {module}: {issues} issues ({high_issues} high severity)")

print("\n3. SPECIFIC ACTIONS NEEDED:")
tdg_remaining = sum(d['tdg_issues'] for d in module_metrics.values())
print(f"   - Total TDG issues in key modules: {tdg_remaining}")
print(f"   - High severity issues to fix: {sum(d['high_severity_issues'] for d in module_metrics.values())}")