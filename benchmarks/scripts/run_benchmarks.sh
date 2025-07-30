#!/bin/bash

# Comprehensive benchmark runner script
# Orchestrates dataset creation, transpiler benchmarking, and report generation

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_DIR")"

# Default values
CATEGORY="all"
COMPARE_WITH=""
DATASETS="small_synthetic,medium_synthetic"
TRANSPILERS="ellex,swc,tsc"
CLEANUP_AFTER=true
GENERATE_REPORT=true
PARALLEL_JOBS=4

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive transpiler benchmark runner for Ellex vs industry standards.

OPTIONS:
    --category CATEGORY       Benchmark category: performance, quality, scalability, all (default: all)
    --compare TRANSPILER      Compare specifically against: swc, tsc, babel, esbuild
    --datasets LIST           Comma-separated dataset names (default: small_synthetic,medium_synthetic)
    --transpilers LIST        Comma-separated transpiler names (default: ellex,swc,tsc)
    --parallel-jobs N         Number of parallel benchmark jobs (default: 4)
    --no-cleanup             Don't cleanup datasets after benchmarking
    --no-report              Don't generate final report
    --help                   Show this help message

EXAMPLES:
    # Run all benchmarks with default settings
    $0

    # Compare Ellex against SWC only
    $0 --compare swc --transpilers ellex,swc

    # Run performance benchmarks on large datasets
    $0 --category performance --datasets large_synthetic,typescript_compiler

    # Quick test run
    $0 --datasets small_synthetic --transpilers ellex,swc --no-cleanup

BENCHMARK CATEGORIES:
    performance    - Compilation time, memory usage, throughput
    quality        - Output correctness, runtime performance, source maps
    scalability    - Performance vs dataset size analysis
    all           - Complete benchmark suite (default)

AVAILABLE DATASETS:
    small_synthetic     - 10 files, ~500 LOC
    medium_synthetic    - 100 files, ~5K LOC  
    large_synthetic     - 500 files, ~25K LOC
    react_real_world    - Real React application
    typescript_compiler - TypeScript compiler source
    lodash_library      - Lodash utility library

AVAILABLE TRANSPILERS:
    ellex      - Ellex natural language transpiler
    swc        - SWC Speedy Web Compiler
    tsc        - TypeScript Compiler
    babel      - Babel JavaScript compiler
    esbuild    - esbuild bundler/transpiler
EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --category)
                CATEGORY="$2"
                shift 2
                ;;
            --compare)
                COMPARE_WITH="$2"
                shift 2
                ;;
            --datasets)
                DATASETS="$2"
                shift 2
                ;;
            --transpilers)
                TRANSPILERS="$2"
                shift 2
                ;;
            --parallel-jobs)
                PARALLEL_JOBS="$2"
                shift 2
                ;;
            --no-cleanup)
                CLEANUP_AFTER=false
                shift
                ;;
            --no-report)
                GENERATE_REPORT=false
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is required but not installed"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is required but not installed"
        exit 1
    fi
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        log_error "Python 3 is required but not installed"
        exit 1
    fi
    
    # Check required Python packages
    python3 -c "import pandas, numpy, matplotlib, seaborn, scipy" 2>/dev/null || {
        log_error "Required Python packages missing. Install with:"
        log_error "pip install pandas numpy matplotlib seaborn scipy"
        exit 1
    }
    
    # Check if Docker daemon is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi
    
    log_info "Prerequisites check passed"
}

# Setup benchmark environment
setup_environment() {
    log_step "Setting up benchmark environment..."
    
    cd "$BENCHMARK_DIR"
    
    # Create necessary directories
    mkdir -p datasets/{synthetic,real-world,npm-packages}
    mkdir -p results
    mkdir -p reports
    mkdir -p monitoring/data
    
    # Build Docker containers
    log_info "Building Docker containers..."
    docker-compose build --parallel
    
    # Wait for containers to be ready
    log_info "Waiting for containers to be ready..."
    sleep 5
    
    log_info "Environment setup complete"
}

# Create datasets
create_datasets() {
    log_step "Creating benchmark datasets..."
    
    IFS=',' read -ra DATASET_ARRAY <<< "$DATASETS"
    
    for dataset in "${DATASET_ARRAY[@]}"; do
        log_info "Creating dataset: $dataset"
        python3 "$SCRIPT_DIR/dataset_manager.py" create --preset "$dataset" || {
            log_warn "Failed to create dataset: $dataset"
            continue
        }
    done
    
    # List created datasets
    log_info "Available datasets:"
    python3 "$SCRIPT_DIR/dataset_manager.py" list
}

# Validate transpiler containers
validate_transpilers() {
    log_step "Validating transpiler containers..."
    
    python3 "$SCRIPT_DIR/benchmark_runner.py" validate || {
        log_error "Transpiler validation failed"
        exit 1
    }
    
    log_info "All transpilers validated successfully"
}

# Run performance benchmarks
run_performance_benchmarks() {
    log_step "Running performance benchmarks..."
    
    IFS=',' read -ra DATASET_ARRAY <<< "$DATASETS"
    IFS=',' read -ra TRANSPILER_ARRAY <<< "$TRANSPILERS"
    
    local benchmark_count=0
    local total_benchmarks=$((${#DATASET_ARRAY[@]} * ${#TRANSPILER_ARRAY[@]}))
    
    for dataset in "${DATASET_ARRAY[@]}"; do
        for transpiler in "${TRANSPILER_ARRAY[@]}"; do
            ((benchmark_count++))
            log_info "Running benchmark $benchmark_count/$total_benchmarks: $transpiler on $dataset"
            
            # Run benchmark with timeout
            timeout 600 python3 "$SCRIPT_DIR/benchmark_runner.py" run \
                --dataset "$dataset" \
                --transpiler "$transpiler" \
                --optimize || {
                log_warn "Benchmark timed out or failed: $transpiler on $dataset"
                continue
            }
        done
    done
    
    log_info "Performance benchmarks completed"
}

# Run quality benchmarks
run_quality_benchmarks() {
    log_step "Running quality benchmarks..."
    
    # Quality benchmarks focus on output correctness and optimization
    IFS=',' read -ra DATASET_ARRAY <<< "$DATASETS"
    IFS=',' read -ra TRANSPILER_ARRAY <<< "$TRANSPILERS"
    
    for dataset in "${DATASET_ARRAY[@]}"; do
        log_info "Quality benchmark on dataset: $dataset"
        
        # Run with different optimization levels
        for opt_level in "false" "true"; do
            for transpiler in "${TRANSPILER_ARRAY[@]}"; do
                log_info "Quality test: $transpiler (optimize=$opt_level) on $dataset"
                
                local cmd_args="--dataset $dataset --transpiler $transpiler"
                if [[ "$opt_level" == "true" ]]; then
                    cmd_args="$cmd_args --optimize --minify"
                fi
                
                timeout 300 python3 "$SCRIPT_DIR/benchmark_runner.py" run $cmd_args || {
                    log_warn "Quality benchmark failed: $transpiler on $dataset"
                }
            done
        done
    done
    
    log_info "Quality benchmarks completed"
}

# Run scalability benchmarks
run_scalability_benchmarks() {
    log_step "Running scalability benchmarks..."
    
    # Create datasets of varying sizes for scalability testing
    local scalability_datasets="small_synthetic,medium_synthetic"
    
    # Try to include large dataset if possible
    if [[ "$DATASETS" == *"large_synthetic"* ]]; then
        scalability_datasets="$scalability_datasets,large_synthetic"
    fi
    
    IFS=',' read -ra DATASET_ARRAY <<< "$scalability_datasets"
    IFS=',' read -ra TRANSPILER_ARRAY <<< "$TRANSPILERS"
    
    for dataset in "${DATASET_ARRAY[@]}"; do
        for transpiler in "${TRANSPILER_ARRAY[@]}"; do
            log_info "Scalability test: $transpiler on $dataset"
            
            # Run multiple times for statistical significance
            for run in {1..3}; do
                timeout 900 python3 "$SCRIPT_DIR/benchmark_runner.py" run \
                    --dataset "$dataset" \
                    --transpiler "$transpiler" || {
                    log_warn "Scalability benchmark failed: run $run"
                }
            done
        done
    done
    
    log_info "Scalability benchmarks completed"
}

# Run specific comparison
run_comparison() {
    local target_transpiler="$1"
    
    log_step "Running comparison: Ellex vs $target_transpiler"
    
    IFS=',' read -ra DATASET_ARRAY <<< "$DATASETS"
    
    for dataset in "${DATASET_ARRAY[@]}"; do
        log_info "Comparing on dataset: $dataset"
        
        # Run Ellex
        python3 "$SCRIPT_DIR/benchmark_runner.py" run \
            --dataset "$dataset" \
            --transpiler ellex \
            --optimize
        
        # Run comparison target
        python3 "$SCRIPT_DIR/benchmark_runner.py" run \
            --dataset "$dataset" \
            --transpiler "$target_transpiler" \
            --optimize
    done
    
    log_info "Comparison benchmarks completed"
}

# Generate analysis report
generate_report() {
    if [[ "$GENERATE_REPORT" != "true" ]]; then
        log_info "Skipping report generation (--no-report specified)"
        return
    fi
    
    log_step "Generating analysis report..."
    
    python3 "$SCRIPT_DIR/analyze_results.py" \
        --results-dir "$BENCHMARK_DIR/results" \
        --reports-dir "$BENCHMARK_DIR/reports"
    
    log_info "Report generated at: $BENCHMARK_DIR/reports/benchmark_report.html"
    
    # Try to open report in browser (if available)
    if command -v open &> /dev/null; then
        open "$BENCHMARK_DIR/reports/benchmark_report.html" 2>/dev/null || true
    elif command -v xdg-open &> /dev/null; then
        xdg-open "$BENCHMARK_DIR/reports/benchmark_report.html" 2>/dev/null || true
    fi
}

# Cleanup datasets
cleanup_datasets() {
    if [[ "$CLEANUP_AFTER" != "true" ]]; then
        log_info "Skipping dataset cleanup (--no-cleanup specified)"
        return
    fi
    
    log_step "Cleaning up datasets..."
    
    python3 "$SCRIPT_DIR/dataset_manager.py" cleanup-all
    
    log_info "Dataset cleanup completed"
}

# Monitor system resources during benchmarks
start_monitoring() {
    log_info "Starting system monitoring..."
    
    # Start Prometheus monitoring in background
    docker-compose up -d monitoring 2>/dev/null || {
        log_warn "Failed to start monitoring - continuing without it"
    }
}

# Stop monitoring
stop_monitoring() {
    log_info "Stopping system monitoring..."
    
    docker-compose stop monitoring 2>/dev/null || true
}

# Main execution function
main() {
    log_info "🚀 Starting Ellex Transpiler Benchmark Suite"
    log_info "Category: $CATEGORY"
    log_info "Datasets: $DATASETS"
    log_info "Transpilers: $TRANSPILERS"
    
    # Setup
    check_prerequisites
    setup_environment
    create_datasets
    validate_transpilers
    start_monitoring
    
    # Run benchmarks based on category
    case "$CATEGORY" in
        performance)
            run_performance_benchmarks
            ;;
        quality)
            run_quality_benchmarks
            ;;
        scalability)
            run_scalability_benchmarks
            ;;
        all)
            run_performance_benchmarks
            run_quality_benchmarks
            run_scalability_benchmarks
            ;;
        *)
            log_error "Unknown category: $CATEGORY"
            exit 1
            ;;
    esac
    
    # Run specific comparison if requested
    if [[ -n "$COMPARE_WITH" ]]; then
        run_comparison "$COMPARE_WITH"
    fi
    
    # Analysis and cleanup
    stop_monitoring
    generate_report
    cleanup_datasets
    
    log_info "✅ Benchmark suite completed successfully!"
    log_info "Results available in: $BENCHMARK_DIR/reports/"
}

# Trap for cleanup on interrupt
cleanup_on_interrupt() {
    log_warn "Benchmark interrupted - cleaning up..."
    stop_monitoring
    docker-compose down 2>/dev/null || true
    exit 1
}

trap cleanup_on_interrupt INT TERM

# Parse arguments and run
parse_args "$@"
main