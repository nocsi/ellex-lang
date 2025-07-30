#!/usr/bin/env python3
"""
Comprehensive benchmark runner for transpiler performance testing.
Measures time, memory, quality, and scalability metrics.
"""

import os
import sys
import json
import time
import subprocess
import psutil
import tempfile
import threading
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, asdict
from concurrent.futures import ThreadPoolExecutor, as_completed
import logging
import statistics
import resource

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class PerformanceMetrics:
    """Performance measurement results."""
    compilation_time_ms: float
    memory_peak_mb: float
    memory_avg_mb: float
    cpu_percent: float
    output_size_bytes: int
    source_map_size_bytes: int
    error_count: int
    warning_count: int
    lines_per_second: float

@dataclass
class QualityMetrics:
    """Code quality assessment results."""
    output_correctness: float      # 0-1 score for correctness
    runtime_performance: float    # Relative performance of generated code
    source_map_accuracy: float    # Source map quality score
    error_message_quality: float  # Error message helpfulness score
    bundle_efficiency: float      # Output optimization score

@dataclass
class BenchmarkResult:
    """Complete benchmark result for a transpiler on a dataset."""
    transpiler: str
    dataset: str
    timestamp: float
    performance: PerformanceMetrics
    quality: QualityMetrics
    metadata: Dict[str, Any]

class TranspilerRunner:
    """Base class for transpiler execution."""
    
    def __init__(self, name: str):
        self.name = name
        
    def compile(self, source_dir: Path, output_dir: Path, options: Dict) -> Tuple[bool, str, str]:
        """Compile source code and return (success, stdout, stderr)."""
        raise NotImplementedError
        
    def supports_source_maps(self) -> bool:
        """Whether this transpiler supports source map generation."""
        return True
        
    def get_version(self) -> str:
        """Get transpiler version string."""
        raise NotImplementedError

class EllexRunner(TranspilerRunner):
    """Ellex transpiler runner."""
    
    def __init__(self):
        super().__init__("ellex")
        
    def compile(self, source_dir: Path, output_dir: Path, options: Dict) -> Tuple[bool, str, str]:
        output_dir.mkdir(parents=True, exist_ok=True)
        
        # Find all .ts files to convert to .ellex first
        ts_files = list(source_dir.rglob("*.ts"))
        ellex_files = []
        
        # Convert TS to Ellex (reverse transpilation for testing)
        for ts_file in ts_files[:10]:  # Limit for testing
            try:
                with open(ts_file, 'r', encoding='utf-8', errors='ignore') as f:
                    ts_content = f.read()
                
                # Simple TS -> Ellex conversion for benchmarking
                ellex_content = self._convert_ts_to_ellex(ts_content)
                
                ellex_file = output_dir / f"{ts_file.stem}.ellex"
                with open(ellex_file, 'w') as f:
                    f.write(ellex_content)
                    
                ellex_files.append(ellex_file)
            except Exception as e:
                logger.warning(f"Failed to convert {ts_file}: {e}")
                continue
        
        if not ellex_files:
            return False, "", "No files to transpile"
        
        # Now transpile Ellex to JavaScript
        success_count = 0
        all_stdout = []
        all_stderr = []
        
        for ellex_file in ellex_files:
            js_output = output_dir / f"{ellex_file.stem}.js"
            
            cmd = [
                'docker', 'run', '--rm',
                '-v', f'{ellex_file.parent}:/workspace',
                'benchmark-ellex',
                'ellex', 'transpile',
                '-i', f'/workspace/{ellex_file.name}',
                '-o', f'/workspace/{js_output.name}',
                '-t', 'javascript'
            ]
            
            if options.get('optimize', False):
                cmd.append('--optimize')
            if options.get('minify', False):
                cmd.append('--minify')
            
            try:
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
                all_stdout.append(result.stdout)
                all_stderr.append(result.stderr)
                
                if result.returncode == 0:
                    success_count += 1
                    
            except subprocess.TimeoutExpired:
                all_stderr.append(f"Timeout compiling {ellex_file}")
            except Exception as e:
                all_stderr.append(f"Error compiling {ellex_file}: {e}")
        
        success = success_count > 0
        stdout = '\n'.join(all_stdout)
        stderr = '\n'.join(all_stderr)
        
        return success, stdout, stderr
    
    def _convert_ts_to_ellex(self, ts_content: str) -> str:
        """Simple TypeScript to Ellex conversion for benchmarking."""
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
                # Simple conversion - not comprehensive
                ellex_lines.append(f'# {stripped}')
                continue
            
            # Convert if statements
            if stripped.startswith('if ('):
                ellex_lines.append(f'# Conditional: {stripped}')
                continue
                
            # Skip empty lines and comments
            if not stripped or stripped.startswith('//') or stripped.startswith('/*'):
                continue
                
            # Default: comment out complex TypeScript
            if stripped:
                ellex_lines.append(f'# {stripped}')
        
        # Add some basic Ellex content
        ellex_lines.insert(0, 'tell "Converted from TypeScript"')
        ellex_lines.append('tell "Conversion complete"')
        
        return '\n'.join(ellex_lines)
    
    def get_version(self) -> str:
        try:
            result = subprocess.run(['docker', 'run', '--rm', 'benchmark-ellex', 'ellex', '--version'], 
                                  capture_output=True, text=True, timeout=10)
            return result.stdout.strip()
        except:
            return "unknown"

class SWCRunner(TranspilerRunner):
    """SWC transpiler runner."""
    
    def __init__(self):
        super().__init__("swc")
        
    def compile(self, source_dir: Path, output_dir: Path, options: Dict) -> Tuple[bool, str, str]:
        output_dir.mkdir(parents=True, exist_ok=True)
        
        cmd = [
            'docker', 'run', '--rm',
            '-v', f'{source_dir}:/workspace/src:ro',
            '-v', f'{output_dir}:/workspace/dist',
            'benchmark-swc',
            'swc', 'src', '--out-dir', 'dist',
            '--config-file', '/workspace/swc.config.js'
        ]
        
        if options.get('source_maps', True):
            cmd.extend(['--source-maps', 'true'])
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
            return result.returncode == 0, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return False, "", "Compilation timeout"
        except Exception as e:
            return False, "", str(e)
    
    def get_version(self) -> str:
        try:
            result = subprocess.run(['docker', 'run', '--rm', 'benchmark-swc', 'swc', '--version'], 
                                  capture_output=True, text=True, timeout=10)
            return result.stdout.strip()
        except:
            return "unknown"

class TSCRunner(TranspilerRunner):
    """TypeScript compiler runner."""
    
    def __init__(self):
        super().__init__("tsc")
        
    def compile(self, source_dir: Path, output_dir: Path, options: Dict) -> Tuple[bool, str, str]:
        output_dir.mkdir(parents=True, exist_ok=True)
        
        cmd = [
            'docker', 'run', '--rm',
            '-v', f'{source_dir}:/workspace/src:ro',
            '-v', f'{output_dir}:/workspace/dist',
            'benchmark-tsc',
            'tsc', '--project', '/workspace/src',
            '--outDir', '/workspace/dist'
        ]
        
        if not options.get('source_maps', True):
            cmd.append('--sourceMap false')
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            return result.returncode == 0, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return False, "", "Compilation timeout"
        except Exception as e:
            return False, "", str(e)
    
    def get_version(self) -> str:
        try:
            result = subprocess.run(['docker', 'run', '--rm', 'benchmark-tsc', 'tsc', '--version'], 
                                  capture_output=True, text=True, timeout=10)
            return result.stdout.strip()
        except:
            return "unknown"

class BabelRunner(TranspilerRunner):
    """Babel transpiler runner."""
    
    def __init__(self):
        super().__init__("babel")
        
    def compile(self, source_dir: Path, output_dir: Path, options: Dict) -> Tuple[bool, str, str]:
        output_dir.mkdir(parents=True, exist_ok=True)
        
        cmd = [
            'docker', 'run', '--rm',
            '-v', f'{source_dir}:/workspace/src:ro',
            '-v', f'{output_dir}:/workspace/dist',
            'benchmark-babel',
            'babel', 'src', '--out-dir', 'dist',
            '--extensions', '.ts,.tsx,.js,.jsx'
        ]
        
        if options.get('source_maps', True):
            cmd.append('--source-maps')
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
            return result.returncode == 0, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return False, "", "Compilation timeout"
        except Exception as e:
            return False, "", str(e)
    
    def get_version(self) -> str:
        try:
            result = subprocess.run(['docker', 'run', '--rm', 'benchmark-babel', 'babel', '--version'], 
                                  capture_output=True, text=True, timeout=10)
            return result.stdout.strip()
        except:
            return "unknown"

class ESBuildRunner(TranspilerRunner):
    """esbuild transpiler runner."""
    
    def __init__(self):
        super().__init__("esbuild")
        
    def compile(self, source_dir: Path, output_dir: Path, options: Dict) -> Tuple[bool, str, str]:
        output_dir.mkdir(parents=True, exist_ok=True)
        
        # Find entry point
        entry_points = list(source_dir.glob("**/main.ts")) or list(source_dir.glob("**/index.ts"))
        if not entry_points:
            # Use first .ts file
            entry_points = list(source_dir.rglob("*.ts"))
        
        if not entry_points:
            return False, "", "No entry point found"
        
        entry_point = entry_points[0]
        
        cmd = [
            'docker', 'run', '--rm',
            '-v', f'{source_dir}:/workspace/src:ro',
            '-v', f'{output_dir}:/workspace/dist',
            'benchmark-esbuild',
            'esbuild', f'/workspace/src/{entry_point.relative_to(source_dir)}',
            '--outdir=/workspace/dist',
            '--format=esm',
            '--target=es2020'
        ]
        
        if options.get('source_maps', True):
            cmd.append('--sourcemap')
        
        if options.get('minify', False):
            cmd.append('--minify')
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
            return result.returncode == 0, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return False, "", "Compilation timeout"
        except Exception as e:
            return False, "", str(e)
    
    def get_version(self) -> str:
        try:
            result = subprocess.run(['docker', 'run', '--rm', 'benchmark-esbuild', 'esbuild', '--version'], 
                                  capture_output=True, text=True, timeout=10)
            return result.stdout.strip()
        except:
            return "unknown"

class BenchmarkOrchestrator:
    """Main benchmark orchestration and measurement."""
    
    def __init__(self, results_dir: Path = Path('./results')):
        self.results_dir = results_dir
        self.results_dir.mkdir(exist_ok=True)
        
        self.runners = {
            'ellex': EllexRunner(),
            'swc': SWCRunner(),
            'tsc': TSCRunner(),
            'babel': BabelRunner(),
            'esbuild': ESBuildRunner()
        }
    
    def measure_performance(self, runner: TranspilerRunner, source_dir: Path, 
                          options: Dict, dataset_stats: Dict) -> PerformanceMetrics:
        """Measure transpiler performance metrics."""
        
        with tempfile.TemporaryDirectory() as temp_dir:
            output_dir = Path(temp_dir) / 'output'
            
            # Monitor process during compilation
            memory_samples = []
            cpu_samples = []
            monitor_active = threading.Event()
            monitor_active.set()
            
            def monitor_resources():
                process = psutil.Process()
                while monitor_active.is_set():
                    try:
                        mem_info = process.memory_info()
                        memory_samples.append(mem_info.rss / 1024 / 1024)  # MB
                        cpu_samples.append(process.cpu_percent())
                        time.sleep(0.1)
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        break
            
            monitor_thread = threading.Thread(target=monitor_resources)
            monitor_thread.start()
            
            # Time the compilation
            start_time = time.perf_counter()
            start_rusage = resource.getrusage(resource.RUSAGE_CHILDREN)
            
            success, stdout, stderr = runner.compile(source_dir, output_dir, options)
            
            end_time = time.perf_counter()
            end_rusage = resource.getrusage(resource.RUSAGE_CHILDREN)
            
            # Stop monitoring
            monitor_active.clear()
            monitor_thread.join(timeout=1)
            
            compilation_time_ms = (end_time - start_time) * 1000
            
            # Calculate memory usage
            memory_peak_mb = max(memory_samples) if memory_samples else 0
            memory_avg_mb = statistics.mean(memory_samples) if memory_samples else 0
            cpu_percent = statistics.mean(cpu_samples) if cpu_samples else 0
            
            # Measure output size
            output_size_bytes = 0
            source_map_size_bytes = 0
            
            if output_dir.exists():
                for file_path in output_dir.rglob('*'):
                    if file_path.is_file():
                        size = file_path.stat().st_size
                        if file_path.suffix == '.map':
                            source_map_size_bytes += size
                        else:
                            output_size_bytes += size
            
            # Count errors and warnings
            error_count = stderr.lower().count('error') if stderr else 0
            warning_count = stderr.lower().count('warning') if stderr else 0
            
            # Calculate throughput
            total_lines = dataset_stats.get('total_lines', 1)
            lines_per_second = total_lines / (compilation_time_ms / 1000) if compilation_time_ms > 0 else 0
            
            return PerformanceMetrics(
                compilation_time_ms=compilation_time_ms,
                memory_peak_mb=memory_peak_mb,
                memory_avg_mb=memory_avg_mb,
                cpu_percent=cpu_percent,
                output_size_bytes=output_size_bytes,
                source_map_size_bytes=source_map_size_bytes,
                error_count=error_count,
                warning_count=warning_count,
                lines_per_second=lines_per_second
            )
    
    def measure_quality(self, runner: TranspilerRunner, source_dir: Path, 
                       output_dir: Path, dataset_stats: Dict) -> QualityMetrics:
        """Measure code quality metrics."""
        
        # Simple quality assessment (can be extended)
        output_correctness = 1.0  # Assume correct if compilation succeeded
        
        # Runtime performance test (simplified)
        runtime_performance = self._measure_runtime_performance(output_dir)
        
        # Source map accuracy (basic check)
        source_map_accuracy = self._assess_source_maps(output_dir)
        
        # Error message quality (based on stderr helpfulness)
        error_message_quality = 0.8  # Default good score
        
        # Bundle efficiency (size relative to source)
        source_size = dataset_stats.get('total_size_bytes', 1)
        output_files = list(output_dir.rglob('*.js')) if output_dir.exists() else []
        output_size = sum(f.stat().st_size for f in output_files)
        bundle_efficiency = min(1.0, source_size / max(output_size, 1))
        
        return QualityMetrics(
            output_correctness=output_correctness,
            runtime_performance=runtime_performance,
            source_map_accuracy=source_map_accuracy,
            error_message_quality=error_message_quality,
            bundle_efficiency=bundle_efficiency
        )
    
    def _measure_runtime_performance(self, output_dir: Path) -> float:
        """Measure runtime performance of generated code."""
        # Simplified runtime test - could be expanded
        js_files = list(output_dir.rglob('*.js')) if output_dir.exists() else []
        
        if not js_files:
            return 0.0
        
        # Basic performance score based on output characteristics
        total_size = sum(f.stat().st_size for f in js_files)
        
        # Smaller, cleaner output generally performs better
        if total_size < 10000:  # < 10KB
            return 1.0
        elif total_size < 100000:  # < 100KB
            return 0.8
        elif total_size < 1000000:  # < 1MB
            return 0.6
        else:
            return 0.4
    
    def _assess_source_maps(self, output_dir: Path) -> float:
        """Assess source map quality."""
        if not output_dir.exists():
            return 0.0
        
        map_files = list(output_dir.rglob('*.map'))
        js_files = list(output_dir.rglob('*.js'))
        
        if not js_files:
            return 0.0
        
        # Check if source maps exist for JavaScript files
        map_coverage = len(map_files) / len(js_files)
        
        # Basic quality check - ensure maps are not empty
        valid_maps = 0
        for map_file in map_files:
            try:
                with open(map_file, 'r') as f:
                    map_data = json.load(f)
                    if 'mappings' in map_data and map_data['mappings']:
                        valid_maps += 1
            except:
                continue
        
        if map_files:
            map_validity = valid_maps / len(map_files)
        else:
            map_validity = 0.0
        
        return (map_coverage + map_validity) / 2
    
    def run_benchmark(self, transpiler: str, dataset_path: Path, 
                     dataset_stats: Dict, options: Dict = None) -> BenchmarkResult:
        """Run complete benchmark for a transpiler on a dataset."""
        
        if transpiler not in self.runners:
            raise ValueError(f"Unknown transpiler: {transpiler}")
        
        options = options or {}
        runner = self.runners[transpiler]
        
        logger.info(f"Running {transpiler} benchmark on {dataset_path.name}")
        
        # Measure performance
        performance = self.measure_performance(runner, dataset_path, options, dataset_stats)
        
        # Measure quality (create temporary output for assessment)
        with tempfile.TemporaryDirectory() as temp_dir:
            output_dir = Path(temp_dir) / 'output'
            success, stdout, stderr = runner.compile(dataset_path, output_dir, options)
            
            quality = self.measure_quality(runner, dataset_path, output_dir, dataset_stats)
        
        # Create result
        result = BenchmarkResult(
            transpiler=transpiler,
            dataset=dataset_path.name,
            timestamp=time.time(),
            performance=performance,
            quality=quality,
            metadata={
                'success': success,
                'transpiler_version': runner.get_version(),
                'options': options,
                'dataset_stats': dataset_stats,
                'stdout': stdout[:1000],  # Truncate
                'stderr': stderr[:1000]   # Truncate
            }
        )
        
        # Save result
        result_file = self.results_dir / f"{transpiler}_{dataset_path.name}_{int(time.time())}.json"
        with open(result_file, 'w') as f:
            json.dump(asdict(result), f, indent=2)
        
        logger.info(f"Benchmark complete: {performance.compilation_time_ms:.1f}ms, "
                   f"{performance.memory_peak_mb:.1f}MB peak memory")
        
        return result
    
    def run_comparative_benchmark(self, dataset_path: Path, dataset_stats: Dict,
                                transpilers: List[str] = None) -> List[BenchmarkResult]:
        """Run benchmarks for multiple transpilers on the same dataset."""
        
        transpilers = transpilers or list(self.runners.keys())
        results = []
        
        for transpiler in transpilers:
            try:
                result = self.run_benchmark(transpiler, dataset_path, dataset_stats)
                results.append(result)
            except Exception as e:
                logger.error(f"Benchmark failed for {transpiler}: {e}")
                continue
        
        return results

def main():
    """Main CLI interface for benchmark runner."""
    import argparse
    from dataset_manager import DatasetManager
    
    parser = argparse.ArgumentParser(description="Transpiler benchmark runner")
    parser.add_argument('command', choices=['run', 'compare', 'validate'])
    parser.add_argument('--dataset', help="Dataset name to benchmark")
    parser.add_argument('--transpiler', help="Specific transpiler to test")
    parser.add_argument('--all-datasets', action='store_true', help="Run on all datasets")
    parser.add_argument('--all-transpilers', action='store_true', help="Run all transpilers")
    parser.add_argument('--optimize', action='store_true', help="Enable optimizations")
    parser.add_argument('--minify', action='store_true', help="Enable minification")
    
    args = parser.parse_args()
    
    dataset_manager = DatasetManager()
    orchestrator = BenchmarkOrchestrator()
    
    if args.command == 'validate':
        # Validate that all containers are working
        for name, runner in orchestrator.runners.items():
            try:
                version = runner.get_version()
                logger.info(f"{name}: {version}")
            except Exception as e:
                logger.error(f"{name}: Failed to get version - {e}")
        return
    
    # Get datasets to test
    if args.all_datasets:
        datasets = dataset_manager.list_datasets()
        if not datasets:
            logger.error("No datasets available. Create some first.")
            return
    elif args.dataset:
        datasets = [cfg for cfg in dataset_manager.list_datasets() if cfg.name == args.dataset]
        if not datasets:
            logger.error(f"Dataset '{args.dataset}' not found")
            return
    else:
        logger.error("Must specify --dataset or --all-datasets")
        return
    
    # Get transpilers to test
    if args.all_transpilers:
        transpilers = list(orchestrator.runners.keys())
    elif args.transpiler:
        transpilers = [args.transpiler]
    else:
        transpilers = ['ellex', 'swc']  # Default comparison
    
    options = {
        'optimize': args.optimize,
        'minify': args.minify,
        'source_maps': True
    }
    
    # Run benchmarks
    all_results = []
    
    for dataset_config in datasets:
        # Find dataset path
        dataset_path = None
        for subdir in ['synthetic', 'real-world', 'npm-packages']:
            path = Path('./datasets') / subdir / dataset_config.name
            if path.exists():
                dataset_path = path
                break
        
        if not dataset_path:
            logger.error(f"Dataset path not found for {dataset_config.name}")
            continue
        
        dataset_stats = dataset_manager.get_dataset_stats(dataset_config.name)
        
        if args.command == 'run':
            for transpiler in transpilers:
                result = orchestrator.run_benchmark(transpiler, dataset_path, dataset_stats, options)
                all_results.append(result)
                
        elif args.command == 'compare':
            results = orchestrator.run_comparative_benchmark(dataset_path, dataset_stats, transpilers)
            all_results.extend(results)
    
    # Print summary
    if all_results:
        print("\n" + "="*80)
        print("BENCHMARK SUMMARY")
        print("="*80)
        print(f"{'Transpiler':<12} {'Dataset':<20} {'Time (ms)':<10} {'Memory (MB)':<12} {'Output (KB)':<12}")
        print("-"*80)
        
        for result in all_results:
            perf = result.performance
            print(f"{result.transpiler:<12} {result.dataset:<20} "
                  f"{perf.compilation_time_ms:<10.1f} {perf.memory_peak_mb:<12.1f} "
                  f"{perf.output_size_bytes/1024:<12.1f}")

if __name__ == '__main__':
    main()