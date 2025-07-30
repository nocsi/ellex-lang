#!/usr/bin/env python3
"""
Automated test dataset generation and management for transpiler benchmarks.
Handles creation, cleanup, and management of test codebases.
"""

import os
import sys
import json
import shutil
import subprocess
import tempfile
import urllib.request
import tarfile
import zipfile
import random
import string
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from concurrent.futures import ThreadPoolExecutor, as_completed
import logging

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class DatasetConfig:
    name: str
    description: str
    size_category: str  # 'small', 'medium', 'large', 'xlarge'
    source_type: str    # 'synthetic', 'github', 'npm_package', 'template'
    parameters: Dict
    expected_loc: int
    cleanup_after_hours: int = 24

@dataclass  
class BenchmarkResult:
    dataset_name: str
    transpiler: str
    compilation_time_ms: float
    memory_peak_mb: float
    output_size_bytes: int
    error_count: int
    warning_count: int

class DatasetManager:
    def __init__(self, datasets_dir: Path = Path('./datasets')):
        self.datasets_dir = datasets_dir
        self.datasets_dir.mkdir(exist_ok=True)
        self.config_file = self.datasets_dir / 'datasets_config.json'
        self.active_datasets: Dict[str, DatasetConfig] = {}
        self.load_config()
        
    def load_config(self):
        """Load dataset configuration from JSON file."""
        if self.config_file.exists():
            with open(self.config_file, 'r') as f:
                config_data = json.load(f)
                for name, data in config_data.items():
                    self.active_datasets[name] = DatasetConfig(**data)
    
    def save_config(self):
        """Save current dataset configuration."""
        config_data = {
            name: {
                'name': cfg.name,
                'description': cfg.description,
                'size_category': cfg.size_category,
                'source_type': cfg.source_type,
                'parameters': cfg.parameters,
                'expected_loc': cfg.expected_loc,
                'cleanup_after_hours': cfg.cleanup_after_hours
            }
            for name, cfg in self.active_datasets.items()
        }
        with open(self.config_file, 'w') as f:
            json.dump(config_data, f, indent=2)
    
    def create_synthetic_dataset(self, config: DatasetConfig) -> Path:
        """Generate synthetic TypeScript/JavaScript codebase."""
        dataset_path = self.datasets_dir / 'synthetic' / config.name
        dataset_path.mkdir(parents=True, exist_ok=True)
        
        params = config.parameters
        num_files = params.get('num_files', 100)
        avg_lines_per_file = params.get('avg_lines_per_file', 50)
        complexity_level = params.get('complexity', 'medium')
        
        logger.info(f"Generating synthetic dataset '{config.name}' with {num_files} files")
        
        # Generate package.json
        self._generate_package_json(dataset_path, config.name)
        
        # Generate tsconfig.json
        self._generate_tsconfig(dataset_path)
        
        # Generate source files
        src_dir = dataset_path / 'src'
        src_dir.mkdir(exist_ok=True)
        
        with ThreadPoolExecutor(max_workers=4) as executor:
            futures = []
            for i in range(num_files):
                future = executor.submit(
                    self._generate_source_file,
                    src_dir / f'module_{i:04d}.ts',
                    avg_lines_per_file,
                    complexity_level,
                    i
                )
                futures.append(future)
            
            for future in as_completed(futures):
                future.result()  # Wait for completion
        
        # Generate main entry point
        self._generate_main_file(src_dir, num_files)
        
        logger.info(f"Generated synthetic dataset at {dataset_path}")
        return dataset_path
    
    def clone_github_repo(self, config: DatasetConfig) -> Path:
        """Clone a GitHub repository for testing."""
        dataset_path = self.datasets_dir / 'real-world' / config.name
        
        if dataset_path.exists():
            shutil.rmtree(dataset_path)
            
        params = config.parameters
        repo_url = params['repo_url']
        branch = params.get('branch', 'main')
        subdir = params.get('subdir', '')
        
        logger.info(f"Cloning {repo_url} (branch: {branch})")
        
        # Clone repository
        subprocess.run([
            'git', 'clone', '--depth', '1', '--branch', branch,
            repo_url, str(dataset_path)
        ], check=True, capture_output=True)
        
        # If subdir specified, move contents up
        if subdir:
            subdir_path = dataset_path / subdir
            if subdir_path.exists():
                temp_dir = dataset_path.parent / f"{config.name}_temp"
                shutil.move(str(subdir_path), str(temp_dir))
                shutil.rmtree(dataset_path)
                shutil.move(str(temp_dir), str(dataset_path))
        
        # Remove .git directory to save space
        git_dir = dataset_path / '.git'
        if git_dir.exists():
            shutil.rmtree(git_dir)
            
        logger.info(f"Cloned repository to {dataset_path}")
        return dataset_path
    
    def download_npm_package(self, config: DatasetConfig) -> Path:
        """Download and extract NPM package for testing."""
        dataset_path = self.datasets_dir / 'npm-packages' / config.name
        dataset_path.mkdir(parents=True, exist_ok=True)
        
        params = config.parameters
        package_name = params['package_name']
        version = params.get('version', 'latest')
        
        logger.info(f"Downloading npm package {package_name}@{version}")
        
        # Download package tarball
        npm_url = f"https://registry.npmjs.org/{package_name}/-/{package_name}-{version}.tgz"
        
        with tempfile.NamedTemporaryFile(suffix='.tgz', delete=False) as tmp_file:
            urllib.request.urlretrieve(npm_url, tmp_file.name)
            
            # Extract tarball
            with tarfile.open(tmp_file.name, 'r:gz') as tar:
                tar.extractall(dataset_path)
        
        os.unlink(tmp_file.name)
        
        # Move package contents up from 'package' subdirectory
        package_dir = dataset_path / 'package'
        if package_dir.exists():
            for item in package_dir.iterdir():
                shutil.move(str(item), str(dataset_path))
            package_dir.rmdir()
            
        logger.info(f"Downloaded npm package to {dataset_path}")
        return dataset_path
    
    def create_dataset(self, config: DatasetConfig) -> Path:
        """Create dataset based on configuration."""
        self.active_datasets[config.name] = config
        self.save_config()
        
        if config.source_type == 'synthetic':
            return self.create_synthetic_dataset(config)
        elif config.source_type == 'github':
            return self.clone_github_repo(config)
        elif config.source_type == 'npm_package':
            return self.download_npm_package(config)
        else:
            raise ValueError(f"Unknown source type: {config.source_type}")
    
    def cleanup_dataset(self, dataset_name: str):
        """Remove a dataset and its configuration."""
        if dataset_name in self.active_datasets:
            config = self.active_datasets[dataset_name]
            
            # Find and remove dataset directory
            for subdir in ['synthetic', 'real-world', 'npm-packages']:
                dataset_path = self.datasets_dir / subdir / dataset_name
                if dataset_path.exists():
                    logger.info(f"Removing dataset {dataset_path}")
                    shutil.rmtree(dataset_path)
            
            # Remove from active datasets
            del self.active_datasets[dataset_name]
            self.save_config()
            
            logger.info(f"Cleaned up dataset '{dataset_name}'")
    
    def cleanup_all(self):
        """Remove all datasets."""
        for dataset_name in list(self.active_datasets.keys()):
            self.cleanup_dataset(dataset_name)
    
    def list_datasets(self) -> List[DatasetConfig]:
        """List all active datasets."""
        return list(self.active_datasets.values())
    
    def get_dataset_stats(self, dataset_name: str) -> Dict:
        """Get statistics about a dataset."""
        if dataset_name not in self.active_datasets:
            return {}
        
        config = self.active_datasets[dataset_name]
        
        # Find dataset path
        dataset_path = None
        for subdir in ['synthetic', 'real-world', 'npm-packages']:
            path = self.datasets_dir / subdir / dataset_name
            if path.exists():
                dataset_path = path
                break
        
        if not dataset_path:
            return {'error': 'Dataset not found'}
        
        stats = {
            'name': dataset_name,
            'path': str(dataset_path),
            'size_category': config.size_category,
            'source_type': config.source_type,
        }
        
        # Count files and lines
        ts_files = list(dataset_path.rglob('*.ts'))
        js_files = list(dataset_path.rglob('*.js'))
        tsx_files = list(dataset_path.rglob('*.tsx'))
        jsx_files = list(dataset_path.rglob('*.jsx'))
        
        all_files = ts_files + js_files + tsx_files + jsx_files
        
        total_lines = 0
        total_size = 0
        
        for file_path in all_files:
            try:
                with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                    lines = len(f.readlines())
                    total_lines += lines
                    total_size += file_path.stat().st_size
            except Exception:
                continue
        
        stats.update({
            'total_files': len(all_files),
            'ts_files': len(ts_files),
            'js_files': len(js_files),
            'tsx_files': len(tsx_files),
            'jsx_files': len(jsx_files),
            'total_lines': total_lines,
            'total_size_bytes': total_size,
            'avg_lines_per_file': total_lines / len(all_files) if all_files else 0
        })
        
        return stats
    
    def _generate_package_json(self, dataset_path: Path, name: str):
        """Generate a package.json file."""
        package_json = {
            "name": name,
            "version": "1.0.0",
            "description": f"Synthetic benchmark dataset: {name}",
            "main": "dist/main.js",
            "scripts": {
                "build": "tsc",
                "dev": "ts-node src/main.ts"
            },
            "devDependencies": {
                "typescript": "^5.3.3",
                "ts-node": "^10.9.2",
                "@types/node": "^20.10.6"
            },
            "dependencies": {
                "lodash": "^4.17.21",
                "axios": "^1.6.2",
                "express": "^4.18.2"
            }
        }
        
        with open(dataset_path / 'package.json', 'w') as f:
            json.dump(package_json, f, indent=2)
    
    def _generate_tsconfig(self, dataset_path: Path):
        """Generate a tsconfig.json file."""
        tsconfig = {
            "compilerOptions": {
                "target": "ES2020",
                "module": "CommonJS",
                "outDir": "./dist",
                "rootDir": "./src",
                "strict": True,
                "esModuleInterop": True,
                "skipLibCheck": True,
                "forceConsistentCasingInFileNames": True,
                "declaration": True,
                "sourceMap": True
            },
            "include": ["src/**/*"],
            "exclude": ["node_modules", "dist"]
        }
        
        with open(dataset_path / 'tsconfig.json', 'w') as f:
            json.dump(tsconfig, f, indent=2)
    
    def _generate_source_file(self, file_path: Path, target_lines: int, complexity: str, module_id: int):
        """Generate a single TypeScript source file."""
        lines = []
        
        # Imports
        lines.append("import { EventEmitter } from 'events';")
        lines.append("import * as fs from 'fs';")
        lines.append("import * as path from 'path';")
        if module_id > 0:
            lines.append(f"import {{ Module{module_id - 1} }} from './module_{module_id - 1:04d}';")
        lines.append("")
        
        # Interface definitions
        lines.append(f"export interface Config{module_id} {{")
        lines.append("  id: number;")
        lines.append("  name: string;")
        lines.append("  enabled: boolean;")
        lines.append("  metadata?: Record<string, any>;")
        lines.append("}")
        lines.append("")
        
        # Class definition
        class_name = f"Module{module_id}"
        lines.append(f"export class {class_name} extends EventEmitter {{")
        lines.append(f"  private config: Config{module_id};")
        lines.append("  private cache: Map<string, any> = new Map();")
        lines.append("")
        
        # Constructor
        lines.append(f"  constructor(config: Config{module_id}) {{")
        lines.append("    super();")
        lines.append("    this.config = config;")
        lines.append("  }")
        lines.append("")
        
        # Methods based on complexity
        if complexity == 'simple':
            methods_count = 2
        elif complexity == 'medium':
            methods_count = 4
        else:  # complex
            methods_count = 6
        
        for method_id in range(methods_count):
            lines.extend(self._generate_method(method_id, complexity))
            lines.append("")
        
        lines.append("}")
        lines.append("")
        
        # Add more content to reach target lines
        while len(lines) < target_lines:
            lines.extend(self._generate_utility_function(len(lines)))
            lines.append("")
        
        # Write file
        with open(file_path, 'w') as f:
            f.write('\n'.join(lines))
    
    def _generate_method(self, method_id: int, complexity: str) -> List[str]:
        """Generate a method with specified complexity."""
        lines = []
        method_name = f"process{method_id}"
        
        if complexity == 'simple':
            lines.append(f"  public {method_name}(data: string): string {{")
            lines.append("    return data.toUpperCase();")
            lines.append("  }")
        elif complexity == 'medium':
            lines.append(f"  public async {method_name}(data: any[]): Promise<any[]> {{")
            lines.append("    const results: any[] = [];")
            lines.append("    for (const item of data) {")
            lines.append("      if (typeof item === 'string') {")
            lines.append("        results.push(item.toLowerCase());")
            lines.append("      } else if (typeof item === 'number') {")
            lines.append("        results.push(item * 2);")
            lines.append("      } else {")
            lines.append("        results.push(JSON.stringify(item));")
            lines.append("      }")
            lines.append("    }")
            lines.append("    return results;")
            lines.append("  }")
        else:  # complex
            lines.append(f"  public async {method_name}<T>(")
            lines.append("    data: T[],")
            lines.append("    transformer: (item: T) => Promise<T>,")
            lines.append("    filter?: (item: T) => boolean")
            lines.append("  ): Promise<T[]> {")
            lines.append("    const cacheKey = `${method_id}_${JSON.stringify(data)}`;")
            lines.append("    if (this.cache.has(cacheKey)) {")
            lines.append("      return this.cache.get(cacheKey);")
            lines.append("    }")
            lines.append("")
            lines.append("    let processed = data;")
            lines.append("    if (filter) {")
            lines.append("      processed = data.filter(filter);")
            lines.append("    }")
            lines.append("")
            lines.append("    const results = await Promise.all(")
            lines.append("      processed.map(async (item) => {")
            lines.append("        try {")
            lines.append("          const result = await transformer(item);")
            lines.append("          this.emit('itemProcessed', { item, result });")
            lines.append("          return result;")
            lines.append("        } catch (error) {")
            lines.append("          this.emit('error', error);")
            lines.append("          return item;")
            lines.append("        }")
            lines.append("      })")
            lines.append("    );")
            lines.append("")
            lines.append("    this.cache.set(cacheKey, results);")
            lines.append("    return results;")
            lines.append("  }")
        
        return lines
    
    def _generate_utility_function(self, seed: int) -> List[str]:
        """Generate a utility function to add more lines."""
        lines = []
        func_name = f"utility{seed % 10}"
        
        lines.append(f"function {func_name}(input: unknown): string {{")
        lines.append("  if (input === null || input === undefined) {")
        lines.append("    return 'null';")
        lines.append("  }")
        lines.append("  if (typeof input === 'object') {")
        lines.append("    return JSON.stringify(input, null, 2);")
        lines.append("  }")
        lines.append("  return String(input);")
        lines.append("}")
        
        return lines
    
    def _generate_main_file(self, src_dir: Path, num_modules: int):
        """Generate main entry point that imports all modules."""
        lines = []
        
        # Imports
        for i in range(num_modules):
            lines.append(f"import {{ Module{i} }} from './module_{i:04d}';")
        
        lines.append("")
        lines.append("async function main() {")
        lines.append("  console.log('Starting benchmark application...');")
        lines.append("")
        
        # Create module instances
        for i in range(min(10, num_modules)):  # Limit to first 10 for main
            lines.append(f"  const module{i} = new Module{i}({{")
            lines.append(f"    id: {i},")
            lines.append(f"    name: 'Module{i}',")
            lines.append("    enabled: true")
            lines.append("  });")
        
        lines.append("")
        lines.append("  console.log('All modules initialized');")
        lines.append("}")
        lines.append("")
        lines.append("if (require.main === module) {")
        lines.append("  main().catch(console.error);")
        lines.append("}")
        
        with open(src_dir / 'main.ts', 'w') as f:
            f.write('\n'.join(lines))

# Predefined dataset configurations
STANDARD_DATASETS = [
    DatasetConfig(
        name="small_synthetic",
        description="Small synthetic TypeScript project (10 files, ~500 LOC)",
        size_category="small",
        source_type="synthetic",
        parameters={
            "num_files": 10,
            "avg_lines_per_file": 50,
            "complexity": "simple"
        },
        expected_loc=500
    ),
    DatasetConfig(
        name="medium_synthetic",
        description="Medium synthetic TypeScript project (100 files, ~5K LOC)",
        size_category="medium",
        source_type="synthetic", 
        parameters={
            "num_files": 100,
            "avg_lines_per_file": 50,
            "complexity": "medium"
        },
        expected_loc=5000
    ),
    DatasetConfig(
        name="large_synthetic",
        description="Large synthetic TypeScript project (500 files, ~25K LOC)",
        size_category="large",
        source_type="synthetic",
        parameters={
            "num_files": 500,
            "avg_lines_per_file": 50,
            "complexity": "complex"
        },
        expected_loc=25000
    ),
    DatasetConfig(
        name="react_real_world",
        description="Real-world React application (Create React App)",
        size_category="medium",
        source_type="github",
        parameters={
            "repo_url": "https://github.com/facebook/create-react-app.git",
            "branch": "main",
            "subdir": "packages/react-scripts/template"
        },
        expected_loc=3000
    ),
    DatasetConfig(
        name="typescript_compiler",
        description="TypeScript compiler source code",
        size_category="xlarge",
        source_type="github",
        parameters={
            "repo_url": "https://github.com/microsoft/TypeScript.git",
            "branch": "main",
            "subdir": "src"
        },
        expected_loc=500000
    ),
    DatasetConfig(
        name="lodash_library",
        description="Lodash utility library",
        size_category="medium",
        source_type="npm_package",
        parameters={
            "package_name": "lodash",
            "version": "4.17.21"
        },
        expected_loc=15000
    )
]

def main():
    """Main CLI interface for dataset management."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Transpiler benchmark dataset manager")
    parser.add_argument('command', choices=['create', 'list', 'stats', 'cleanup', 'cleanup-all'])
    parser.add_argument('--dataset', help="Dataset name")
    parser.add_argument('--preset', choices=[cfg.name for cfg in STANDARD_DATASETS], 
                       help="Use a predefined dataset configuration")
    parser.add_argument('--all-presets', action='store_true',
                       help="Create all predefined datasets")
    
    args = parser.parse_args()
    
    manager = DatasetManager()
    
    if args.command == 'create':
        if args.all_presets:
            for config in STANDARD_DATASETS:
                try:
                    path = manager.create_dataset(config)
                    logger.info(f"Created dataset '{config.name}' at {path}")
                except Exception as e:
                    logger.error(f"Failed to create dataset '{config.name}': {e}")
        elif args.preset:
            config = next(cfg for cfg in STANDARD_DATASETS if cfg.name == args.preset)
            path = manager.create_dataset(config)
            logger.info(f"Created dataset '{config.name}' at {path}")
        else:
            logger.error("Must specify --preset or --all-presets")
            
    elif args.command == 'list':
        datasets = manager.list_datasets()
        if not datasets:
            print("No active datasets")
        else:
            print(f"{'Name':<20} {'Size':<10} {'Type':<15} {'Description'}")
            print("-" * 80)
            for dataset in datasets:
                print(f"{dataset.name:<20} {dataset.size_category:<10} {dataset.source_type:<15} {dataset.description}")
                
    elif args.command == 'stats':
        if not args.dataset:
            logger.error("Must specify --dataset")
            return
        stats = manager.get_dataset_stats(args.dataset)
        if 'error' in stats:
            logger.error(stats['error'])
        else:
            print(json.dumps(stats, indent=2))
            
    elif args.command == 'cleanup':
        if not args.dataset:
            logger.error("Must specify --dataset")
            return
        manager.cleanup_dataset(args.dataset)
        
    elif args.command == 'cleanup-all':
        manager.cleanup_all()
        logger.info("All datasets cleaned up")

if __name__ == '__main__':
    main()