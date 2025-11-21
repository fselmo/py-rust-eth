# rust-pyspec-utils

Rust acceleration library for Ethereum execution and consensus specs.

## Overview

This library provides high-performance Rust implementations of computationally intensive Ethereum operations, designed to work seamlessly with the Python execution-specs project.

## Features

- **State Root Computation**: Fast state root calculation using Rust's alloy-trie library
- **Python Bindings**: Seamless integration with Python via PyO3
- **Drop-in Replacement**: Compatible with execution-specs State objects

## Installation

```bash
# Development installation with maturin
maturin develop
```

## Usage

```python
import rust_pyspec_utils
from ethereum.forks.frontier.state import State

# Create a state
state = State()

# Compute state root
root = rust_pyspec_utils.state_root(state)
```

## Development

This project uses:
- Rust with PyO3 for Python bindings
- Alloy libraries for Ethereum primitives and trie operations
- Maturin for building and packaging

## License

MIT