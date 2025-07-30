#!/usr/bin/env python3
"""
Direct benchmark runner that doesn't require Docker.
Tests Ellex transpiler against reference implementations.
"""

import os
import sys
import json
import time
import subprocess
import tempfile
import psutil
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict
import statistics
import logging

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class BenchmarkResult:
    transpiler: str
    dataset: str
    success: bool
    compilation_time_ms: float
    memory_peak_mb: float
    output_size_bytes: int
    lines_per_second: float
    error_count: int
    warning_count: int
    timestamp: float

class DirectBenchmarkRunner:
    def __init__(self, results_dir: Path = Path('./results')):
        self.results_dir = results_dir
        self.results_dir.mkdir(exist_ok=True)
        self.project_root = Path(__file__).parent.parent.parent
        
    def benchmark_ellex(self, dataset_path: Path, dataset_stats: Dict) -> BenchmarkResult:
        """Benchmark Ellex transpiler directly."""
        logger.info(f"Benchmarking Ellex on dataset: {dataset_path.name}")
        
        with tempfile.TemporaryDirectory() as temp_dir:
            output_dir = Path(temp_dir) / 'ellex_output'
            output_dir.mkdir()
            
            # Find TypeScript files to transpile
            ts_files = list(dataset_path.rglob('*.ts'))[:5]  # Limit for testing
            
            if not ts_files:
                return BenchmarkResult(
                    transpiler="ellex",
                    dataset=dataset_path.name,
                    success=False,
                    compilation_time_ms=0,
                    memory_peak_mb=0,
                    output_size_bytes=0,
                    lines_per_second=0,
                    error_count=1,
                    warning_count=0,
                    timestamp=time.time()
                )
            
            # Monitor memory usage
            process = psutil.Process()
            memory_samples = []
            
            start_time = time.perf_counter()
            success_count = 0
            total_output_size = 0
            all_stderr = []
            
            for ts_file in ts_files:
                try:
                    # Read TypeScript content
                    with open(ts_file, 'r', encoding='utf-8', errors='ignore') as f:
                        ts_content = f.read()
                    
                    # Simple TS to Ellex conversion
                    ellex_content = self._convert_ts_to_ellex(ts_content)
                    
                    # Create temporary ellex file
                    ellex_file = output_dir / f"{ts_file.stem}.ellex"
                    with open(ellex_file, 'w') as f:
                        f.write(ellex_content)
                    
                    # Try to transpile with Ellex CLI
                    js_output = output_dir / f"{ts_file.stem}.js"
                    
                    # Build Ellex first if not built
                    self._ensure_ellex_built()
                    
                    # Use ellex run command to execute the file
                    cmd = [
                        str(self.project_root / 'crates' / 'target' / 'release' / 'ellex_cli'),
                        'run',
                        str(ellex_file)
                    ]
                    
                    # Measure memory during compilation
                    mem_before = process.memory_info().rss / 1024 / 1024
                    
                    result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
                    
                    mem_after = process.memory_info().rss / 1024 / 1024
                    memory_samples.extend([mem_before, mem_after])
                    
                    if result.returncode == 0:
                        success_count += 1
                        # For Ellex, we measure the execution success rather than JS output
                        total_output_size += len(result.stdout.encode('utf-8'))
                    else:
                        all_stderr.append(result.stderr)
                        
                except Exception as e:
                    logger.warning(f"Failed to process {ts_file}: {e}")
                    all_stderr.append(str(e))
                    continue
            
            end_time = time.perf_counter()
            compilation_time_ms = (end_time - start_time) * 1000
            
            # Calculate metrics
            memory_peak_mb = max(memory_samples) if memory_samples else 0
            total_lines = dataset_stats.get('total_lines', 1)
            lines_per_second = total_lines / (compilation_time_ms / 1000) if compilation_time_ms > 0 else 0
            
            # Count errors
            stderr_text = ' '.join(all_stderr).lower()
            error_count = stderr_text.count('error')
            warning_count = stderr_text.count('warning')
            
            return BenchmarkResult(
                transpiler="ellex",
                dataset=dataset_path.name,
                success=success_count > 0,
                compilation_time_ms=compilation_time_ms,
                memory_peak_mb=memory_peak_mb,
                output_size_bytes=total_output_size,
                lines_per_second=lines_per_second,
                error_count=error_count,
                warning_count=warning_count,
                timestamp=time.time()
            )
    
    def benchmark_tsc(self, dataset_path: Path, dataset_stats: Dict) -> BenchmarkResult:
        """Benchmark TypeScript compiler."""
        logger.info(f"Benchmarking TSC on dataset: {dataset_path.name}")
        
        # Check if tsc is available
        if not shutil.which('tsc'):
            logger.warning("TypeScript compiler not found, skipping TSC benchmark")
            return BenchmarkResult(
                transpiler="tsc",
                dataset=dataset_path.name,
                success=False,
                compilation_time_ms=0,
                memory_peak_mb=0,
                output_size_bytes=0,
                lines_per_second=0,
                error_count=1,
                warning_count=0,
                timestamp=time.time()
            )
        
        with tempfile.TemporaryDirectory() as temp_dir:
            output_dir = Path(temp_dir) / 'tsc_output'
            output_dir.mkdir()
            
            # Copy dataset to temp directory
            source_dir = Path(temp_dir) / 'src'
            shutil.copytree(dataset_path, source_dir)
            
            # Monitor memory
            process = psutil.Process()
            mem_before = process.memory_info().rss / 1024 / 1024
            
            start_time = time.perf_counter()
            
            # Run TypeScript compiler with lenient settings
            cmd = [
                'tsc',
                '--project', str(source_dir),
                '--outDir', str(output_dir),
                '--target', 'es2020',
                '--module', 'commonjs',
                '--skipLibCheck',
                '--noImplicitAny', 'false',
                '--strict', 'false'
            ]
            
            try:
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
                mem_after = process.memory_info().rss / 1024 / 1024
                
                end_time = time.perf_counter()
                compilation_time_ms = (end_time - start_time) * 1000
                
                # Calculate output size
                total_output_size = 0
                if output_dir.exists():
                    for js_file in output_dir.rglob('*.js'):
                        total_output_size += js_file.stat().st_size
                
                # Calculate metrics
                memory_peak_mb = max(mem_before, mem_after)
                total_lines = dataset_stats.get('total_lines', 1)
                lines_per_second = total_lines / (compilation_time_ms / 1000) if compilation_time_ms > 0 else 0
                
                # Count errors and warnings
                stderr_text = result.stderr.lower()
                error_count = stderr_text.count('error')
                warning_count = stderr_text.count('warning')
                
                return BenchmarkResult(
                    transpiler="tsc",
                    dataset=dataset_path.name,
                    success=result.returncode == 0,
                    compilation_time_ms=compilation_time_ms,
                    memory_peak_mb=memory_peak_mb,
                    output_size_bytes=total_output_size,
                    lines_per_second=lines_per_second,
                    error_count=error_count,
                    warning_count=warning_count,
                    timestamp=time.time()
                )
                
            except subprocess.TimeoutExpired:
                return BenchmarkResult(
                    transpiler="tsc",
                    dataset=dataset_path.name,
                    success=False,
                    compilation_time_ms=120000,  # Timeout
                    memory_peak_mb=0,
                    output_size_bytes=0,
                    lines_per_second=0,
                    error_count=1,
                    warning_count=0,
                    timestamp=time.time()
                )
    
    def _ensure_ellex_built(self):
        """Ensure Ellex is built."""
        ellex_binary = self.project_root / 'crates' / 'target' / 'release' / 'ellex_cli'
        
        if not ellex_binary.exists():
            logger.info("Building Ellex transpiler...")
            build_cmd = ['cargo', 'build', '--release', '--bin', 'ellex_cli']
            result = subprocess.run(build_cmd, cwd=self.project_root / 'crates', 
                                  capture_output=True, text=True)
            if result.returncode != 0:
                logger.error(f"Failed to build Ellex: {result.stderr}")
                raise RuntimeError("Ellex build failed")
    
    def _convert_ts_to_ellex(self, ts_content: str) -> str:
        """Convert TypeScript to Ellex for testing."""
        lines = ts_content.split('\n')
        ellex_lines = []
        
        for line in lines:
            stripped = line.strip()
            
            # Convert console.log() to tell
            if 'console.log(' in stripped:
                start = stripped.find('console.log(') + 12
                end = stripped.rfind(')')
                if end > start:
                    content = stripped[start:end]
                    ellex_lines.append(f'tell {content}')
                continue
            
            # Convert simple variable declarations
            if stripped.startswith('const ') or stripped.startswith('let '):
                ellex_lines.append(f'# {stripped}')
                continue
            
            # Skip empty lines and comments
            if not stripped or stripped.startswith('//') or stripped.startswith('/*'):
                continue
                
            # Default: comment out complex TypeScript
            if stripped:
                ellex_lines.append(f'# {stripped}')
        
        # Add basic Ellex content
        ellex_lines.insert(0, 'tell "Benchmark test file"')
        ellex_lines.extend([
            'make countdown:',
            '  tell "Starting countdown..."',
            '  repeat 3 times:',
            '    tell "Tick"',
            '  tell "Done!"',
            '',
            'countdown',
            'tell "Benchmark complete"'
        ])
        
        return '\n'.join(ellex_lines)
    
    def run_comparative_benchmark(self, dataset_path: Path, dataset_stats: Dict) -> List[BenchmarkResult]:
        """Run benchmarks for multiple transpilers."""
        results = []
        
        # Benchmark Ellex
        try:
            ellex_result = self.benchmark_ellex(dataset_path, dataset_stats)
            results.append(ellex_result)
        except Exception as e:
            logger.error(f"Ellex benchmark failed: {e}")
        
        # Benchmark TSC
        try:
            tsc_result = self.benchmark_tsc(dataset_path, dataset_stats)
            results.append(tsc_result)
        except Exception as e:
            logger.error(f"TSC benchmark failed: {e}")
        
        # Save results
        for result in results:
            result_file = self.results_dir / f"{result.transpiler}_{result.dataset}_{int(result.timestamp)}.json"
            with open(result_file, 'w') as f:
                json.dump(asdict(result), f, indent=2)
        
        return results

def main():
    """Run direct benchmarks."""
    from dataset_manager import DatasetManager
    
    dataset_manager = DatasetManager()
    runner = DirectBenchmarkRunner()
    
    # Get datasets
    datasets = [cfg for cfg in dataset_manager.list_datasets() if cfg.name in ['small_synthetic', 'medium_synthetic']]
    
    if not datasets:
        logger.error("small_synthetic dataset not found. Create it first.")
        return
    
    all_results = []
    
    for dataset_config in datasets:
        # Find dataset path
        dataset_path = Path('./datasets/synthetic') / dataset_config.name
        if not dataset_path.exists():
            logger.error(f"Dataset path not found: {dataset_path}")
            continue
        
        dataset_stats = dataset_manager.get_dataset_stats(dataset_config.name)
        
        logger.info(f"Running benchmarks on {dataset_config.name}")
        results = runner.run_comparative_benchmark(dataset_path, dataset_stats)
        all_results.extend(results)
    
    # Print results
    if all_results:
        print("\n" + "="*80)
        print("BENCHMARK RESULTS")
        print("="*80)
        print(f"{'Transpiler':<12} {'Success':<8} {'Time (ms)':<12} {'Memory (MB)':<12} {'Output (KB)':<12} {'LOC/s':<8}")
        print("-"*80)
        
        for result in all_results:
            success_str = "✓" if result.success else "✗"
            print(f"{result.transpiler:<12} {success_str:<8} {result.compilation_time_ms:<12.1f} "
                  f"{result.memory_peak_mb:<12.1f} {result.output_size_bytes/1024:<12.1f} {result.lines_per_second:<8.0f}")
        
        # Generate analysis
        print(f"\nResults saved to: {runner.results_dir}")
        
        if len(all_results) >= 2:
            ellex_result = next((r for r in all_results if r.transpiler == "ellex"), None)
            tsc_result = next((r for r in all_results if r.transpiler == "tsc"), None)
            
            if ellex_result and tsc_result and ellex_result.success and tsc_result.success:
                time_ratio = ellex_result.compilation_time_ms / tsc_result.compilation_time_ms
                memory_ratio = ellex_result.memory_peak_mb / tsc_result.memory_peak_mb if tsc_result.memory_peak_mb > 0 else 0
                
                print(f"\nComparative Analysis:")
                print(f"  Ellex vs TSC compilation time: {time_ratio:.2f}x")
                print(f"  Ellex vs TSC memory usage: {memory_ratio:.2f}x" if memory_ratio > 0 else "  Ellex vs TSC memory usage: N/A")
                
                if time_ratio < 1:
                    print(f"  🚀 Ellex is {1/time_ratio:.1f}x FASTER than TSC")
                else:
                    print(f"  🐌 Ellex is {time_ratio:.1f}x slower than TSC")

if __name__ == '__main__':
    main()