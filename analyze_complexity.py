#!/usr/bin/env python3
"""
PAIML Complexity Analysis Tool for Rust Codebases
Analyzes cyclomatic complexity, coupling metrics, and technical debt indicators
"""

import json
import re
import os
from datetime import datetime
from collections import defaultdict

class RustComplexityAnalyzer:
    def __init__(self):
        self.files = []
        self.tdg_issues = []
        self.complexity_scores = {}
        
    def analyze_file(self, filepath):
        """Analyze a single Rust file for complexity metrics"""
        with open(filepath, 'r') as f:
            content = f.read()
            
        metrics = {
            'path': filepath,
            'lines': len(content.splitlines()),
            'functions': self.count_functions(content),
            'cyclomatic_complexity': self.calculate_cyclomatic(content),
            'coupling': self.analyze_coupling(content),
            'nesting_depth': self.calculate_max_nesting(content),
            'todo_count': len(re.findall(r'TODO|FIXME|HACK|XXX', content)),
            'unsafe_blocks': len(re.findall(r'unsafe\s*\{', content)),
            'unwrap_count': len(re.findall(r'\.unwrap\(\)', content)),
            'panic_count': len(re.findall(r'panic!|unreachable!|unimplemented!', content)),
            'clone_count': len(re.findall(r'\.clone\(\)', content)),
            'technical_debt_indicators': self.find_tdg_issues(content, filepath)
        }
        
        return metrics
    
    def count_functions(self, content):
        """Count number of functions in file"""
        # Match function definitions
        fn_pattern = r'(?:pub\s+)?(?:async\s+)?fn\s+\w+'
        return len(re.findall(fn_pattern, content))
    
    def calculate_cyclomatic(self, content):
        """Calculate cyclomatic complexity"""
        # Basic approximation: count decision points
        complexity = 1  # Base complexity
        
        # Decision points
        complexity += len(re.findall(r'\bif\b', content))
        complexity += len(re.findall(r'\belse\s+if\b', content))
        complexity += len(re.findall(r'\bmatch\b', content))
        complexity += len(re.findall(r'\bwhile\b', content))
        complexity += len(re.findall(r'\bfor\b', content))
        complexity += len(re.findall(r'\?', content))  # ? operator
        complexity += len(re.findall(r'\|\|', content))  # logical OR
        complexity += len(re.findall(r'&&', content))  # logical AND
        
        # Match arms add complexity
        match_arms = re.findall(r'=>', content)
        complexity += max(0, len(match_arms) - len(re.findall(r'\bmatch\b', content)))
        
        return complexity
    
    def analyze_coupling(self, content):
        """Analyze coupling metrics"""
        coupling = {
            'imports': len(re.findall(r'^use\s+', content, re.MULTILINE)),
            'external_crate_deps': len(re.findall(r'use\s+(?!crate|self|super)', content)),
            'trait_impls': len(re.findall(r'impl\s+\w+\s+for\s+', content)),
            'generic_params': len(re.findall(r'<[^>]+>', content)),
        }
        coupling['total'] = sum(coupling.values())
        return coupling
    
    def calculate_max_nesting(self, content):
        """Calculate maximum nesting depth"""
        max_depth = 0
        current_depth = 0
        
        for char in content:
            if char == '{':
                current_depth += 1
                max_depth = max(max_depth, current_depth)
            elif char == '}':
                current_depth = max(0, current_depth - 1)
                
        return max_depth
    
    def find_tdg_issues(self, content, filepath):
        """Find Technical Debt Gauge (TDG) issues"""
        issues = []
        
        # High complexity functions (heuristic based on length)
        functions = re.finditer(r'fn\s+(\w+)[^{]*\{', content)
        for match in functions:
            fn_name = match.group(1)
            fn_start = match.end()
            
            # Find matching closing brace
            brace_count = 1
            pos = fn_start
            while brace_count > 0 and pos < len(content):
                if content[pos] == '{':
                    brace_count += 1
                elif content[pos] == '}':
                    brace_count -= 1
                pos += 1
            
            fn_content = content[fn_start:pos]
            fn_lines = len(fn_content.splitlines())
            
            if fn_lines > 50:
                issues.append({
                    'type': 'high_complexity_function',
                    'severity': 'high' if fn_lines > 100 else 'medium',
                    'location': f"{filepath}::{fn_name}",
                    'message': f"Function '{fn_name}' has {fn_lines} lines (threshold: 50)",
                    'metrics': {
                        'lines': fn_lines,
                        'cyclomatic': self.calculate_cyclomatic(fn_content)
                    }
                })
        
        # Deeply nested code
        if self.calculate_max_nesting(content) > 5:
            issues.append({
                'type': 'deep_nesting',
                'severity': 'medium',
                'location': filepath,
                'message': f"File has nesting depth of {self.calculate_max_nesting(content)} (threshold: 5)"
            })
        
        # Too many dependencies
        imports = len(re.findall(r'^use\s+', content, re.MULTILINE))
        if imports > 15:
            issues.append({
                'type': 'high_coupling',
                'severity': 'medium',
                'location': filepath,
                'message': f"File has {imports} imports (threshold: 15)"
            })
        
        # Error handling issues
        unwraps = len(re.findall(r'\.unwrap\(\)', content))
        if unwraps > 5:
            issues.append({
                'type': 'excessive_unwraps',
                'severity': 'high',
                'location': filepath,
                'message': f"File contains {unwraps} unwrap() calls (threshold: 5)"
            })
            
        return issues
    
    def analyze_directory(self, directory):
        """Analyze all Rust files in directory"""
        for root, _, files in os.walk(directory):
            for file in files:
                if file.endswith('.rs'):
                    filepath = os.path.join(root, file)
                    try:
                        metrics = self.analyze_file(filepath)
                        self.files.append(metrics)
                        
                        # Collect TDG issues
                        for issue in metrics['technical_debt_indicators']:
                            self.tdg_issues.append(issue)
                            
                    except Exception as e:
                        print(f"Error analyzing {filepath}: {e}")
    
    def calculate_summary(self):
        """Calculate summary statistics"""
        if not self.files:
            return {}
            
        total_lines = sum(f['lines'] for f in self.files)
        total_functions = sum(f['functions'] for f in self.files)
        total_complexity = sum(f['cyclomatic_complexity'] for f in self.files)
        
        return {
            'total_files': len(self.files),
            'total_lines': total_lines,
            'total_functions': total_functions,
            'average_file_size': total_lines / len(self.files),
            'average_complexity': total_complexity / len(self.files) if self.files else 0,
            'total_todos': sum(f['todo_count'] for f in self.files),
            'total_unsafe': sum(f['unsafe_blocks'] for f in self.files),
            'total_unwraps': sum(f['unwrap_count'] for f in self.files),
            'total_panics': sum(f['panic_count'] for f in self.files),
            'tdg_issues_count': len(self.tdg_issues),
            'tdg_severity_breakdown': self.get_severity_breakdown()
        }
    
    def get_severity_breakdown(self):
        """Get breakdown of TDG issues by severity"""
        breakdown = defaultdict(int)
        for issue in self.tdg_issues:
            breakdown[issue['severity']] += 1
        return dict(breakdown)
    
    def find_hotspots(self):
        """Identify complexity hotspots"""
        hotspots = []
        
        for file in self.files:
            score = (
                file['cyclomatic_complexity'] * 2 +
                file['nesting_depth'] * 3 +
                file['coupling']['total'] +
                file['unwrap_count'] * 5 +
                file['panic_count'] * 10
            )
            
            if score > 50:
                hotspots.append({
                    'file': file['path'],
                    'complexity_score': score,
                    'cyclomatic': file['cyclomatic_complexity'],
                    'nesting': file['nesting_depth'],
                    'coupling': file['coupling']['total'],
                    'risk_factors': {
                        'unwraps': file['unwrap_count'],
                        'panics': file['panic_count'],
                        'unsafe': file['unsafe_blocks']
                    }
                })
        
        return sorted(hotspots, key=lambda x: x['complexity_score'], reverse=True)
    
    def generate_report(self):
        """Generate comprehensive analysis report"""
        summary = self.calculate_summary()
        hotspots = self.find_hotspots()
        
        report = {
            'metadata': {
                'analyzed_at': datetime.now().isoformat(),
                'tool': 'PAIML Rust Complexity Analyzer',
                'version': '1.0.0'
            },
            'summary': summary,
            'hotspots': hotspots[:10],  # Top 10 hotspots
            'tdg_issues': self.tdg_issues,
            'detailed_metrics': self.files
        }
        
        return report

# Run analysis
analyzer = RustComplexityAnalyzer()
analyzer.analyze_directory('rash/src')

report = analyzer.generate_report()

# Save report
with open('paiml-analysis.json', 'w') as f:
    json.dump(report, f, indent=2)

# Print summary
print("PAIML Complexity Analysis Complete")
print("=" * 50)
print(f"Total Files Analyzed: {report['summary']['total_files']}")
print(f"Total Lines of Code: {report['summary']['total_lines']}")
print(f"Average Complexity: {report['summary']['average_complexity']:.2f}")
print(f"TDG Issues Found: {report['summary']['tdg_issues_count']}")
print(f"  - High Severity: {report['summary']['tdg_severity_breakdown'].get('high', 0)}")
print(f"  - Medium Severity: {report['summary']['tdg_severity_breakdown'].get('medium', 0)}")
print(f"  - Low Severity: {report['summary']['tdg_severity_breakdown'].get('low', 0)}")
print("\nTop Complexity Hotspots:")
for i, hotspot in enumerate(report['hotspots'][:5], 1):
    print(f"  {i}. {hotspot['file']} (score: {hotspot['complexity_score']})")