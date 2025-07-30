#!/usr/bin/env python3
"""
Statistical analysis and reporting for transpiler benchmarks.
Generates comprehensive reports comparing performance, quality, and scalability.
"""

import os
import sys
import json
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass
import statistics
import scipy.stats as stats
from datetime import datetime
import logging

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Set style for better-looking plots
plt.style.use('seaborn-v0_8')
sns.set_palette("husl")

@dataclass
class BenchmarkSummary:
    """Summary statistics for a transpiler."""
    transpiler: str
    datasets_tested: int
    avg_compilation_time_ms: float
    std_compilation_time_ms: float
    avg_memory_peak_mb: float
    std_memory_peak_mb: float
    avg_output_size_kb: float
    avg_lines_per_second: float
    success_rate: float
    quality_score: float

class ResultsAnalyzer:
    """Analyze and compare transpiler benchmark results."""
    
    def __init__(self, results_dir: Path = Path('./results')):
        self.results_dir = results_dir
        self.reports_dir = Path('./reports')
        self.reports_dir.mkdir(exist_ok=True)
        
    def load_results(self) -> pd.DataFrame:
        """Load all benchmark results into a DataFrame."""
        results = []
        
        for result_file in self.results_dir.glob('*.json'):
            try:
                with open(result_file, 'r') as f:
                    data = json.load(f)
                    
                # Flatten the nested structure
                flattened = {
                    'transpiler': data['transpiler'],
                    'dataset': data['dataset'],
                    'timestamp': data['timestamp'],
                    'success': data['metadata']['success'],
                    'transpiler_version': data['metadata']['transpiler_version'],
                    
                    # Performance metrics
                    'compilation_time_ms': data['performance']['compilation_time_ms'],
                    'memory_peak_mb': data['performance']['memory_peak_mb'],
                    'memory_avg_mb': data['performance']['memory_avg_mb'],
                    'cpu_percent': data['performance']['cpu_percent'],
                    'output_size_bytes': data['performance']['output_size_bytes'],
                    'source_map_size_bytes': data['performance']['source_map_size_bytes'],
                    'error_count': data['performance']['error_count'],
                    'warning_count': data['performance']['warning_count'],
                    'lines_per_second': data['performance']['lines_per_second'],
                    
                    # Quality metrics
                    'output_correctness': data['quality']['output_correctness'],
                    'runtime_performance': data['quality']['runtime_performance'],
                    'source_map_accuracy': data['quality']['source_map_accuracy'],
                    'error_message_quality': data['quality']['error_message_quality'],
                    'bundle_efficiency': data['quality']['bundle_efficiency'],
                    
                    # Dataset info
                    'dataset_lines': data['metadata']['dataset_stats'].get('total_lines', 0),
                    'dataset_files': data['metadata']['dataset_stats'].get('total_files', 0),
                    'dataset_size_bytes': data['metadata']['dataset_stats'].get('total_size_bytes', 0),
                }
                
                results.append(flattened)
                
            except Exception as e:
                logger.warning(f"Failed to load {result_file}: {e}")
                continue
        
        if not results:
            logger.error("No valid results found")
            return pd.DataFrame()
        
        df = pd.DataFrame(results)
        df['datetime'] = pd.to_datetime(df['timestamp'], unit='s')
        df['output_size_kb'] = df['output_size_bytes'] / 1024
        df['dataset_size_kb'] = df['dataset_size_bytes'] / 1024
        
        return df
    
    def generate_summary_stats(self, df: pd.DataFrame) -> List[BenchmarkSummary]:
        """Generate summary statistics for each transpiler."""
        summaries = []
        
        for transpiler in df['transpiler'].unique():
            trans_data = df[df['transpiler'] == transpiler]
            
            # Calculate quality score (weighted average of quality metrics)
            quality_cols = ['output_correctness', 'runtime_performance', 'source_map_accuracy', 
                          'error_message_quality', 'bundle_efficiency']
            quality_score = trans_data[quality_cols].mean().mean()
            
            summary = BenchmarkSummary(
                transpiler=transpiler,
                datasets_tested=len(trans_data['dataset'].unique()),
                avg_compilation_time_ms=trans_data['compilation_time_ms'].mean(),
                std_compilation_time_ms=trans_data['compilation_time_ms'].std(),
                avg_memory_peak_mb=trans_data['memory_peak_mb'].mean(),
                std_memory_peak_mb=trans_data['memory_peak_mb'].std(),
                avg_output_size_kb=trans_data['output_size_kb'].mean(),
                avg_lines_per_second=trans_data['lines_per_second'].mean(),
                success_rate=trans_data['success'].mean(),
                quality_score=quality_score
            )
            
            summaries.append(summary)
        
        return summaries
    
    def create_performance_plots(self, df: pd.DataFrame):
        """Create performance comparison plots."""
        
        # Compilation time comparison
        plt.figure(figsize=(12, 8))
        
        plt.subplot(2, 2, 1)
        sns.boxplot(data=df, x='transpiler', y='compilation_time_ms')
        plt.title('Compilation Time Comparison')
        plt.ylabel('Time (ms)')
        plt.xticks(rotation=45)
        
        plt.subplot(2, 2, 2)
        sns.boxplot(data=df, x='transpiler', y='memory_peak_mb')
        plt.title('Peak Memory Usage Comparison')
        plt.ylabel('Memory (MB)')
        plt.xticks(rotation=45)
        
        plt.subplot(2, 2, 3)
        sns.boxplot(data=df, x='transpiler', y='lines_per_second')
        plt.title('Throughput Comparison')
        plt.ylabel('Lines per Second')
        plt.xticks(rotation=45)
        
        plt.subplot(2, 2, 4)
        sns.boxplot(data=df, x='transpiler', y='output_size_kb')
        plt.title('Output Size Comparison')
        plt.ylabel('Size (KB)')
        plt.xticks(rotation=45)
        
        plt.tight_layout()
        plt.savefig(self.reports_dir / 'performance_comparison.png', dpi=300, bbox_inches='tight')
        plt.close()
        
        logger.info("Generated performance comparison plots")
    
    def create_scalability_plots(self, df: pd.DataFrame):
        """Create scalability analysis plots."""
        
        plt.figure(figsize=(15, 10))
        
        # Time vs dataset size
        plt.subplot(2, 3, 1)
        for transpiler in df['transpiler'].unique():
            trans_data = df[df['transpiler'] == transpiler]
            plt.scatter(trans_data['dataset_lines'], trans_data['compilation_time_ms'], 
                       label=transpiler, alpha=0.7)
        plt.xlabel('Dataset Lines of Code')
        plt.ylabel('Compilation Time (ms)')
        plt.title('Compilation Time vs Dataset Size')
        plt.legend()
        plt.xscale('log')
        plt.yscale('log')
        
        # Memory vs dataset size
        plt.subplot(2, 3, 2)
        for transpiler in df['transpiler'].unique():
            trans_data = df[df['transpiler'] == transpiler]
            plt.scatter(trans_data['dataset_lines'], trans_data['memory_peak_mb'], 
                       label=transpiler, alpha=0.7)
        plt.xlabel('Dataset Lines of Code')
        plt.ylabel('Peak Memory (MB)')
        plt.title('Memory Usage vs Dataset Size')
        plt.legend()
        plt.xscale('log')
        
        # Output size vs input size
        plt.subplot(2, 3, 3)
        for transpiler in df['transpiler'].unique():
            trans_data = df[df['transpiler'] == transpiler]
            plt.scatter(trans_data['dataset_size_kb'], trans_data['output_size_kb'], 
                       label=transpiler, alpha=0.7)
        plt.xlabel('Input Size (KB)')
        plt.ylabel('Output Size (KB)')
        plt.title('Output vs Input Size')
        plt.legend()
        
        # Throughput vs dataset size
        plt.subplot(2, 3, 4)
        for transpiler in df['transpiler'].unique():
            trans_data = df[df['transpiler'] == transpiler]
            plt.scatter(trans_data['dataset_lines'], trans_data['lines_per_second'], 
                       label=transpiler, alpha=0.7)
        plt.xlabel('Dataset Lines of Code')
        plt.ylabel('Lines per Second')
        plt.title('Throughput vs Dataset Size')
        plt.legend()
        plt.xscale('log')
        
        # Quality vs dataset size
        plt.subplot(2, 3, 5)
        quality_cols = ['output_correctness', 'runtime_performance', 'source_map_accuracy', 
                       'error_message_quality', 'bundle_efficiency']
        df['quality_score'] = df[quality_cols].mean(axis=1)
        
        for transpiler in df['transpiler'].unique():
            trans_data = df[df['transpiler'] == transpiler]
            plt.scatter(trans_data['dataset_lines'], trans_data['quality_score'], 
                       label=transpiler, alpha=0.7)
        plt.xlabel('Dataset Lines of Code')
        plt.ylabel('Quality Score')
        plt.title('Quality vs Dataset Size')
        plt.legend()
        plt.xscale('log')
        
        # Success rate by transpiler
        plt.subplot(2, 3, 6)
        success_rates = df.groupby('transpiler')['success'].mean()
        success_rates.plot(kind='bar')
        plt.title('Success Rate by Transpiler')
        plt.ylabel('Success Rate')
        plt.xticks(rotation=45)
        
        plt.tight_layout()
        plt.savefig(self.reports_dir / 'scalability_analysis.png', dpi=300, bbox_inches='tight')
        plt.close()
        
        logger.info("Generated scalability analysis plots")
    
    def create_quality_radar_chart(self, df: pd.DataFrame):
        """Create radar chart comparing quality metrics."""
        
        quality_metrics = ['output_correctness', 'runtime_performance', 'source_map_accuracy', 
                          'error_message_quality', 'bundle_efficiency']
        
        # Calculate average quality scores per transpiler
        quality_data = df.groupby('transpiler')[quality_metrics].mean()
        
        # Set up radar chart
        angles = np.linspace(0, 2 * np.pi, len(quality_metrics), endpoint=False).tolist()
        angles += angles[:1]  # Complete the circle
        
        fig, ax = plt.subplots(figsize=(10, 10), subplot_kw=dict(projection='polar'))
        
        colors = plt.cm.Set1(np.linspace(0, 1, len(quality_data)))
        
        for i, (transpiler, scores) in enumerate(quality_data.iterrows()):
            values = scores.tolist()
            values += values[:1]  # Complete the circle
            
            ax.plot(angles, values, 'o-', linewidth=2, label=transpiler, color=colors[i])
            ax.fill(angles, values, alpha=0.25, color=colors[i])
        
        # Customize the chart
        ax.set_xticks(angles[:-1])
        ax.set_xticklabels([metric.replace('_', ' ').title() for metric in quality_metrics])
        ax.set_ylim(0, 1)
        ax.set_yticks([0.2, 0.4, 0.6, 0.8, 1.0])
        ax.set_yticklabels(['0.2', '0.4', '0.6', '0.8', '1.0'])
        ax.grid(True)
        
        plt.legend(loc='upper right', bbox_to_anchor=(1.2, 1))
        plt.title('Quality Metrics Comparison', size=16, fontweight='bold', pad=20)
        
        plt.savefig(self.reports_dir / 'quality_radar_chart.png', dpi=300, bbox_inches='tight')
        plt.close()
        
        logger.info("Generated quality radar chart")
    
    def perform_statistical_tests(self, df: pd.DataFrame) -> Dict[str, Any]:
        """Perform statistical significance tests."""
        results = {}
        
        transpilers = df['transpiler'].unique()
        if len(transpilers) < 2:
            return results
        
        # Compilation time comparison
        groups = [df[df['transpiler'] == t]['compilation_time_ms'].dropna() for t in transpilers]
        if all(len(g) > 1 for g in groups):
            try:
                statistic, p_value = stats.kruskal(*groups)
                results['compilation_time_kruskal'] = {
                    'statistic': statistic,
                    'p_value': p_value,
                    'significant': p_value < 0.05
                }
            except Exception as e:
                logger.warning(f"Failed to perform Kruskal-Wallis test: {e}")
        
        # Pairwise comparisons for compilation time
        results['pairwise_comparisons'] = {}
        for i, t1 in enumerate(transpilers):
            for t2 in transpilers[i+1:]:
                group1 = df[df['transpiler'] == t1]['compilation_time_ms'].dropna()
                group2 = df[df['transpiler'] == t2]['compilation_time_ms'].dropna()
                
                if len(group1) > 1 and len(group2) > 1:
                    try:
                        statistic, p_value = stats.mannwhitneyu(group1, group2, alternative='two-sided')
                        results['pairwise_comparisons'][f'{t1}_vs_{t2}'] = {
                            'statistic': statistic,
                            'p_value': p_value,
                            'significant': p_value < 0.05,
                            'median_diff': group1.median() - group2.median()
                        }
                    except Exception as e:
                        logger.warning(f"Failed pairwise test {t1} vs {t2}: {e}")
        
        return results
    
    def generate_html_report(self, df: pd.DataFrame, summaries: List[BenchmarkSummary], 
                           stats_results: Dict[str, Any]):
        """Generate comprehensive HTML report."""
        
        html_content = f"""
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Transpiler Benchmark Report</title>
            <style>
                body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; 
                       line-height: 1.6; margin: 0; padding: 20px; background-color: #f5f5f5; }}
                .container {{ max-width: 1200px; margin: 0 auto; background-color: white; 
                            padding: 30px; border-radius: 10px; box-shadow: 0 0 20px rgba(0,0,0,0.1); }}
                h1 {{ color: #333; text-align: center; border-bottom: 3px solid #4CAF50; padding-bottom: 10px; }}
                h2 {{ color: #4CAF50; border-left: 4px solid #4CAF50; padding-left: 15px; }}
                .summary-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); 
                               gap: 20px; margin: 20px 0; }}
                .summary-card {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
                               color: white; padding: 20px; border-radius: 10px; }}
                .summary-card h3 {{ margin-top: 0; }}
                .metric {{ display: flex; justify-content: space-between; margin: 10px 0; }}
                .metric-value {{ font-weight: bold; }}
                table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
                th, td {{ border: 1px solid #ddd; padding: 12px; text-align: left; }}
                th {{ background-color: #4CAF50; color: white; }}
                tr:nth-child(even) {{ background-color: #f2f2f2; }}
                .best-score {{ background-color: #c8e6c9; font-weight: bold; }}
                .chart-container {{ text-align: center; margin: 30px 0; }}
                .chart-container img {{ max-width: 100%; height: auto; border: 1px solid #ddd; 
                                      border-radius: 5px; }}
                .stats-section {{ background-color: #f9f9f9; padding: 20px; border-radius: 5px; 
                                margin: 20px 0; }}
                .significant {{ color: #e53e3e; font-weight: bold; }}
                .not-significant {{ color: #38a169; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🚀 Transpiler Benchmark Report</h1>
                <p style="text-align: center; color: #666; font-size: 18px;">
                    Generated on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
                </p>
                
                <h2>📊 Executive Summary</h2>
                <div class="summary-grid">
        """
        
        # Add summary cards for each transpiler
        for summary in summaries:
            html_content += f"""
                    <div class="summary-card">
                        <h3>{summary.transpiler.upper()}</h3>
                        <div class="metric">
                            <span>Avg. Compilation Time:</span>
                            <span class="metric-value">{summary.avg_compilation_time_ms:.1f} ms</span>
                        </div>
                        <div class="metric">
                            <span>Avg. Memory Usage:</span>
                            <span class="metric-value">{summary.avg_memory_peak_mb:.1f} MB</span>
                        </div>
                        <div class="metric">
                            <span>Throughput:</span>
                            <span class="metric-value">{summary.avg_lines_per_second:.0f} LOC/s</span>
                        </div>
                        <div class="metric">
                            <span>Success Rate:</span>
                            <span class="metric-value">{summary.success_rate:.1%}</span>
                        </div>
                        <div class="metric">
                            <span>Quality Score:</span>
                            <span class="metric-value">{summary.quality_score:.2f}/1.0</span>
                        </div>
                    </div>
            """
        
        # Detailed comparison table
        html_content += f"""
                </div>
                
                <h2>📈 Detailed Performance Comparison</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Transpiler</th>
                            <th>Avg. Time (ms)</th>
                            <th>Std. Dev (ms)</th>
                            <th>Avg. Memory (MB)</th>
                            <th>Throughput (LOC/s)</th>
                            <th>Success Rate</th>
                            <th>Quality Score</th>
                        </tr>
                    </thead>
                    <tbody>
        """
        
        # Find best performers for highlighting
        best_time = min(s.avg_compilation_time_ms for s in summaries)
        best_memory = min(s.avg_memory_peak_mb for s in summaries)
        best_throughput = max(s.avg_lines_per_second for s in summaries)
        best_quality = max(s.quality_score for s in summaries)
        
        for summary in sorted(summaries, key=lambda x: x.avg_compilation_time_ms):
            html_content += f"""
                        <tr>
                            <td><strong>{summary.transpiler}</strong></td>
                            <td class="{'best-score' if summary.avg_compilation_time_ms == best_time else ''}">{summary.avg_compilation_time_ms:.1f}</td>
                            <td>{summary.std_compilation_time_ms:.1f}</td>
                            <td class="{'best-score' if summary.avg_memory_peak_mb == best_memory else ''}">{summary.avg_memory_peak_mb:.1f}</td>
                            <td class="{'best-score' if summary.avg_lines_per_second == best_throughput else ''}">{summary.avg_lines_per_second:.0f}</td>
                            <td>{summary.success_rate:.1%}</td>
                            <td class="{'best-score' if summary.quality_score == best_quality else ''}">{summary.quality_score:.3f}</td>
                        </tr>
            """
        
        html_content += """
                    </tbody>
                </table>
                
                <h2>📊 Performance Visualizations</h2>
                <div class="chart-container">
                    <h3>Performance Metrics Comparison</h3>
                    <img src="performance_comparison.png" alt="Performance Comparison Charts">
                </div>
                
                <div class="chart-container">
                    <h3>Scalability Analysis</h3>
                    <img src="scalability_analysis.png" alt="Scalability Analysis Charts">
                </div>
                
                <div class="chart-container">
                    <h3>Quality Metrics Radar Chart</h3>
                    <img src="quality_radar_chart.png" alt="Quality Radar Chart">
                </div>
        """
        
        # Statistical analysis section
        if stats_results:
            html_content += """
                <h2>🧪 Statistical Analysis</h2>
                <div class="stats-section">
            """
            
            if 'compilation_time_kruskal' in stats_results:
                kruskal = stats_results['compilation_time_kruskal']
                significance = 'significant' if kruskal['significant'] else 'not-significant'
                html_content += f"""
                    <h3>Overall Comparison (Kruskal-Wallis Test)</h3>
                    <p>Testing whether compilation times differ significantly across transpilers:</p>
                    <ul>
                        <li>Test Statistic: {kruskal['statistic']:.3f}</li>
                        <li>P-value: {kruskal['p_value']:.6f}</li>
                        <li>Result: <span class="{significance}">{'Significant' if kruskal['significant'] else 'Not Significant'}</span> difference found</li>
                    </ul>
                """
            
            if 'pairwise_comparisons' in stats_results:
                html_content += """
                    <h3>Pairwise Comparisons (Mann-Whitney U Tests)</h3>
                    <table>
                        <thead>
                            <tr>
                                <th>Comparison</th>
                                <th>P-value</th>
                                <th>Significance</th>
                                <th>Median Difference (ms)</th>
                            </tr>
                        </thead>
                        <tbody>
                """
                
                for comparison, result in stats_results['pairwise_comparisons'].items():
                    significance = 'significant' if result['significant'] else 'not-significant'
                    html_content += f"""
                            <tr>
                                <td>{comparison.replace('_', ' vs ')}</td>
                                <td>{result['p_value']:.6f}</td>
                                <td><span class="{significance}">{'Significant' if result['significant'] else 'Not Significant'}</span></td>
                                <td>{result['median_diff']:.1f}</td>
                            </tr>
                    """
                
                html_content += """
                        </tbody>
                    </table>
                """
            
            html_content += "</div>"
        
        # Conclusions and recommendations
        fastest_transpiler = min(summaries, key=lambda x: x.avg_compilation_time_ms).transpiler
        most_efficient_memory = min(summaries, key=lambda x: x.avg_memory_peak_mb).transpiler
        highest_quality = max(summaries, key=lambda x: x.quality_score).transpiler
        
        html_content += f"""
                <h2>🎯 Key Findings & Recommendations</h2>
                <div class="stats-section">
                    <h3>Performance Leaders</h3>
                    <ul>
                        <li><strong>Fastest Compilation:</strong> {fastest_transpiler}</li>
                        <li><strong>Most Memory Efficient:</strong> {most_efficient_memory}</li>
                        <li><strong>Highest Quality Output:</strong> {highest_quality}</li>
                    </ul>
                    
                    <h3>Recommendations</h3>
                    <ul>
                        <li><strong>For Development:</strong> Use {fastest_transpiler} for fastest iteration cycles</li>
                        <li><strong>For Production:</strong> Consider {highest_quality} for best output quality</li>
                        <li><strong>For Resource-Constrained Environments:</strong> {most_efficient_memory} uses memory most efficiently</li>
                    </ul>
                    
                    <h3>Ellex Performance Analysis</h3>
        """
        
        # Add specific analysis for Ellex
        ellex_summary = next((s for s in summaries if s.transpiler == 'ellex'), None)
        if ellex_summary:
            html_content += f"""
                    <p><strong>Ellex Transpiler Results:</strong></p>
                    <ul>
                        <li>Compilation Time: {ellex_summary.avg_compilation_time_ms:.1f} ms (avg)</li>
                        <li>Memory Usage: {ellex_summary.avg_memory_peak_mb:.1f} MB (avg)</li>
                        <li>Success Rate: {ellex_summary.success_rate:.1%}</li>
                        <li>Quality Score: {ellex_summary.quality_score:.3f}/1.0</li>
                    </ul>
                    
                    <p><strong>Competitive Analysis:</strong></p>
                    <ul>
            """
            
            # Compare Ellex to others
            for other in summaries:
                if other.transpiler != 'ellex':
                    time_ratio = ellex_summary.avg_compilation_time_ms / other.avg_compilation_time_ms
                    memory_ratio = ellex_summary.avg_memory_peak_mb / other.avg_memory_peak_mb
                    
                    html_content += f"""
                        <li><strong>vs {other.transpiler}:</strong> 
                            {time_ratio:.1f}x compilation time, 
                            {memory_ratio:.1f}x memory usage</li>
                    """
            
            html_content += "</ul>"
        else:
            html_content += "<p><em>Ellex results not available in this benchmark run.</em></p>"
        
        html_content += """
                </div>
                
                <hr style="margin: 40px 0;">
                <p style="text-align: center; color: #666;">
                    Generated by Ellex Transpiler Benchmark Suite | 
                    <a href="https://github.com/nocsi/ellex-language">GitHub Repository</a>
                </p>
            </div>
        </body>
        </html>
        """
        
        # Save HTML report
        with open(self.reports_dir / 'benchmark_report.html', 'w') as f:
            f.write(html_content)
        
        logger.info("Generated comprehensive HTML report")
    
    def generate_json_report(self, df: pd.DataFrame, summaries: List[BenchmarkSummary]):
        """Generate machine-readable JSON report."""
        
        report_data = {
            'metadata': {
                'generated_at': datetime.now().isoformat(),
                'total_benchmarks': len(df),
                'transpilers_tested': list(df['transpiler'].unique()),
                'datasets_tested': list(df['dataset'].unique())
            },
            'summaries': [
                {
                    'transpiler': s.transpiler,
                    'datasets_tested': s.datasets_tested,
                    'performance': {
                        'avg_compilation_time_ms': s.avg_compilation_time_ms,
                        'std_compilation_time_ms': s.std_compilation_time_ms,
                        'avg_memory_peak_mb': s.avg_memory_peak_mb,
                        'std_memory_peak_mb': s.std_memory_peak_mb,
                        'avg_output_size_kb': s.avg_output_size_kb,
                        'avg_lines_per_second': s.avg_lines_per_second
                    },
                    'quality': {
                        'success_rate': s.success_rate,
                        'quality_score': s.quality_score
                    }
                }
                for s in summaries
            ],
            'detailed_results': df.to_dict('records')
        }
        
        with open(self.reports_dir / 'benchmark_report.json', 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        logger.info("Generated JSON report")
    
    def analyze_and_report(self):
        """Main analysis pipeline."""
        logger.info("Starting benchmark analysis...")
        
        # Load results
        df = self.load_results()
        if df.empty:
            logger.error("No results to analyze")
            return
        
        logger.info(f"Loaded {len(df)} benchmark results")
        
        # Generate summary statistics
        summaries = self.generate_summary_stats(df)
        
        # Create visualizations
        self.create_performance_plots(df)
        self.create_scalability_plots(df)
        self.create_quality_radar_chart(df)
        
        # Perform statistical tests
        stats_results = self.perform_statistical_tests(df)
        
        # Generate reports
        self.generate_html_report(df, summaries, stats_results)
        self.generate_json_report(df, summaries)
        
        logger.info(f"Analysis complete. Reports generated in {self.reports_dir}")
        
        # Print summary to console
        print("\n" + "="*80)
        print("BENCHMARK ANALYSIS SUMMARY")
        print("="*80)
        
        for summary in sorted(summaries, key=lambda x: x.avg_compilation_time_ms):
            print(f"\n{summary.transpiler.upper()}:")
            print(f"  Average Compilation Time: {summary.avg_compilation_time_ms:.1f} ± {summary.std_compilation_time_ms:.1f} ms")
            print(f"  Average Memory Usage: {summary.avg_memory_peak_mb:.1f} ± {summary.std_memory_peak_mb:.1f} MB")
            print(f"  Throughput: {summary.avg_lines_per_second:.0f} lines/second")
            print(f"  Success Rate: {summary.success_rate:.1%}")
            print(f"  Quality Score: {summary.quality_score:.3f}/1.0")

def main():
    """Main CLI interface."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Analyze transpiler benchmark results")
    parser.add_argument('--results-dir', default='./results', help='Results directory')
    parser.add_argument('--reports-dir', default='./reports', help='Reports output directory')
    
    args = parser.parse_args()
    
    analyzer = ResultsAnalyzer(Path(args.results_dir))
    analyzer.reports_dir = Path(args.reports_dir)
    analyzer.reports_dir.mkdir(exist_ok=True)
    
    analyzer.analyze_and_report()

if __name__ == '__main__':
    main()