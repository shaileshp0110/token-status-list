# OSS-Fuzz Integration

This directory contains the configuration files needed to integrate this project with [Google's OSS-Fuzz](https://google.github.io/oss-fuzz/) continuous fuzzing service.

## Overview

OSS-Fuzz is Google's free, continuous fuzzing service for open source projects. It runs fuzzing tests 24/7 on Google's infrastructure and automatically reports bugs found.

**Official Documentation**: [Integrating a Rust project with OSS-Fuzz](https://google.github.io/oss-fuzz/getting-started/new-project-guide/rust-lang/)

## Project Structure

This project follows the standard Rust fuzzing setup:

- **`fuzz/`** (in project root): Contains fuzz targets and Cargo.toml
  - `fuzz/Cargo.toml`: Fuzzing dependencies
  - `fuzz/fuzz_targets/encoder.rs`: Fuzz target for encoder/decoder

- **`oss-fuzz/`** (this directory): Files to submit to google/oss-fuzz repository
  - `Dockerfile`: Container definition for OSS-Fuzz build environment
  - `build.sh`: Script to build fuzz targets
  - `project.yaml`: Project metadata and configuration

## How to Submit to OSS-Fuzz

### Step 1: Fork and Clone oss-fuzz

```bash
git clone https://github.com/google/oss-fuzz.git
cd oss-fuzz
```

### Step 2: Create Project Directory

```bash
mkdir -p projects/vc-status-list
```

### Step 3: Copy Configuration Files

```bash
# From your project root
cp oss-fuzz/Dockerfile projects/vc-status-list/
cp oss-fuzz/build.sh projects/vc-status-list/
cp oss-fuzz/project.yaml projects/vc-status-list/

# Make build.sh executable
chmod +x projects/vc-status-list/build.sh
```

### Step 4: Test Locally (Optional but Recommended)

Test the integration before submitting:

```bash
# Build the Docker image
python3 infra/helper.py build_image vc-status-list

# Build the fuzzers
python3 infra/helper.py build_fuzzers vc-status-list

# Run a fuzzer
python3 infra/helper.py run_fuzzer vc-status-list encoder
```

### Step 5: Submit Pull Request

1. Commit your changes:
   ```bash
   git add projects/vc-status-list/
   git commit -m "Add vc-status-list project"
   ```

2. Push to your fork and create a pull request to google/oss-fuzz

## Requirements

- ✅ Project is open source
- ✅ Project has a public GitHub repository
- ✅ Project has fuzz targets in `fuzz/fuzz_targets/`
- ✅ Project uses `cargo-fuzz` for building fuzzers
- ✅ Project is actively maintained

## Key Configuration Details

### Dockerfile
- Uses `gcr.io/oss-fuzz-base/base-builder-rust` base image
- Clones the repository from GitHub
- Installs any necessary system dependencies

### build.sh
- Uses `cargo fuzz build -O --debug-assertions` to build fuzzers
  - `-O`: Release mode for performance
  - `--debug-assertions`: Enables additional runtime checks
- Automatically copies all fuzz targets from `fuzz/fuzz_targets/` to `$OUT/`

### project.yaml
- `language: rust`: Specifies Rust language
- `sanitizers: [address]`: Uses AddressSanitizer
- `fuzzing_engines: [libfuzzer]`: Uses libFuzzer engine

## Benefits

- **Free continuous fuzzing** on Google's infrastructure
- **Automatic bug reporting** via GitHub issues
- **Security vulnerability detection**
- **Improved code quality**
- **Better Scorecard ratings** (Fuzzing check)

## Local Testing

You can test fuzzers locally using cargo-fuzz:

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run a fuzzer
cd fuzz
cargo fuzz run encoder -- -max_total_time=60
```

## More Information

- [OSS-Fuzz Documentation](https://google.github.io/oss-fuzz/)
- [Rust Integration Guide](https://google.github.io/oss-fuzz/getting-started/new-project-guide/rust-lang/)
- [OSS-Fuzz GitHub Repository](https://github.com/google/oss-fuzz)

