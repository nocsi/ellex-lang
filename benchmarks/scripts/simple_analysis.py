#!/usr/bin/env python3
"""
Simple analysis script for direct benchmark results.
"""

import json
import glob
from pathlib import Path
import statistics

def analyze_results():
    """Analyze benchmark results and create summary."""
    results_dir = Path('./results')
    result_files = list(results_dir.glob('*.json'))
    
    if not result_files:
        print("No results found")
        return
    
    results = []
    for file_path in result_files:
        try:
            with open(file_path, 'r') as f:
                data = json.load(f)
                results.append(data)
        except Exception as e:
            print(f"Failed to load {file_path}: {e}")
            continue
    
    if not results:
        print("No valid results found")
        return
    
    # Group by transpiler
    transpilers = {}
    for result in results:
        transpiler = result['transpiler']
        if transpiler not in transpilers:
            transpilers[transpiler] = []
        transpilers[transpiler].append(result)
    
    print("="*80)
    print("TRANSPILER BENCHMARK ANALYSIS")
    print("="*80)
    
    summary_stats = {}
    
    for transpiler, results_list in transpilers.items():
        successful_results = [r for r in results_list if r['success']]
        
        if not successful_results:
            print(f"\n{transpiler.upper()}: No successful runs")
            continue
        
        times = [r['compilation_time_ms'] for r in successful_results]
        memories = [r['memory_peak_mb'] for r in successful_results]
        throughputs = [r['lines_per_second'] for r in successful_results]
        
        avg_time = statistics.mean(times)
        std_time = statistics.stdev(times) if len(times) > 1 else 0
        avg_memory = statistics.mean(memories)
        avg_throughput = statistics.mean(throughputs)
        success_rate = len(successful_results) / len(results_list)
        
        summary_stats[transpiler] = {
            'avg_time_ms': avg_time,
            'std_time_ms': std_time,
            'avg_memory_mb': avg_memory,
            'avg_throughput_lps': avg_throughput,
            'success_rate': success_rate,
            'total_runs': len(results_list),
            'successful_runs': len(successful_results)
        }
        
        print(f"\n{transpiler.upper()} PERFORMANCE:")
        print(f"  Total Runs: {len(results_list)}")
        print(f"  Successful: {len(successful_results)} ({success_rate:.1%})")
        print(f"  Avg Compilation Time: {avg_time:.1f} ± {std_time:.1f} ms")
        print(f"  Avg Memory Usage: {avg_memory:.1f} MB")
        print(f"  Avg Throughput: {avg_throughput:.0f} lines/second")
    
    # Comparative analysis
    if len(summary_stats) >= 2:
        print(f"\n{'='*80}")
        print("COMPARATIVE ANALYSIS")
        print("="*80)
        
        transpiler_names = list(summary_stats.keys())
        
        for i, t1 in enumerate(transpiler_names):
            for t2 in transpiler_names[i+1:]:
                s1 = summary_stats[t1]
                s2 = summary_stats[t2]
                
                if s1['success_rate'] > 0 and s2['success_rate'] > 0:
                    time_ratio = s1['avg_time_ms'] / s2['avg_time_ms']
                    memory_ratio = s1['avg_memory_mb'] / s2['avg_memory_mb']
                    throughput_ratio = s1['avg_throughput_lps'] / s2['avg_throughput_lps']
                    
                    print(f"\n{t1.upper()} vs {t2.upper()}:")
                    print(f"  Time Ratio: {time_ratio:.2f}x ({t1} / {t2})")
                    print(f"  Memory Ratio: {memory_ratio:.2f}x")
                    print(f"  Throughput Ratio: {throughput_ratio:.2f}x")
                    
                    if time_ratio < 1:
                        print(f"  🚀 {t1.upper()} is {1/time_ratio:.1f}x FASTER")
                    else:
                        print(f"  🐌 {t1.upper()} is {time_ratio:.1f}x slower")
                    
                    if throughput_ratio > 1:
                        print(f"  📈 {t1.upper()} has {throughput_ratio:.1f}x higher throughput")
    
    # Dataset performance breakdown
    print(f"\n{'='*80}")
    print("DATASET PERFORMANCE BREAKDOWN")
    print("="*80)
    
    datasets = {}
    for result in results:
        dataset = result['dataset']
        if dataset not in datasets:
            datasets[dataset] = {}
        transpiler = result['transpiler']
        if transpiler not in datasets[dataset]:
            datasets[dataset][transpiler] = []
        datasets[dataset][transpiler].append(result)
    
    for dataset_name, dataset_results in datasets.items():
        print(f"\nDataset: {dataset_name}")
        print("-" * 40)
        
        for transpiler, results_list in dataset_results.items():
            successful = [r for r in results_list if r['success']]
            if successful:
                avg_time = statistics.mean([r['compilation_time_ms'] for r in successful])
                avg_throughput = statistics.mean([r['lines_per_second'] for r in successful])
                print(f"  {transpiler:<10}: {avg_time:6.1f}ms  {avg_throughput:8.0f} LOC/s")
    
    # Save summary report
    reports_dir = Path('./reports')
    reports_dir.mkdir(exist_ok=True)
    
    with open(reports_dir / 'benchmark_summary.json', 'w') as f:
        json.dump({
            'summary_stats': summary_stats,
            'datasets': {k: {t: len(r) for t, r in v.items()} for k, v in datasets.items()},
            'total_benchmarks': len(results)
        }, f, indent=2)
    
    print(f"\n📊 Summary report saved to: {reports_dir / 'benchmark_summary.json'}")
    print("✅ Analysis complete!")

if __name__ == '__main__':
    analyze_results()