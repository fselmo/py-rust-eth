"""Tests for rust_pyspec_utils state_root function compatibility with Python State"""

import sys
import os
import rust_pyspec_utils
from pathlib import Path

# Add the execution-specs to the path if available
# Use environment variable or try to find relative to this repo
exec_specs_path = os.environ.get("EXECUTION_SPECS_PATH")
if exec_specs_path:
    exec_specs_path = Path(exec_specs_path) / "src"
else:
    # Try to find it relative to this repo (assumes they're siblings)
    exec_specs_path = Path(__file__).parent.parent.parent / "execution-specs" / "src"

if exec_specs_path.exists():
    sys.path.insert(0, str(exec_specs_path))


def test_rust_pyspec_utils_state_root_with_empty_state():
    """Test that rust_pyspec_utils.state_root works with an empty Python State object."""
    from ethereum.forks.frontier.state import State
    from ethereum.forks.frontier.state import state_root as python_state_root

    # Create an empty state
    state = State()

    # Compute state root using rust_pyspec_utils
    rust_root = rust_pyspec_utils.state_root(state)

    # Compute state root using Python implementation
    python_root = python_state_root(state)

    # Both should return the same empty state root
    assert rust_root == python_root
    assert rust_root == b'\x56\xe8\x1f\x17\x1b\xcc\x55\xa6\xff\x83\x45\xe6\x92\xc0\xf8\x6e\x5b\x48\xe0\x1b\x99\x6c\xad\xc0\x01\x62\x2f\xb5\xe3\x63\xb4\x21'


def test_rust_pyspec_utils_state_root_with_account():
    """Test that rust_pyspec_utils.state_root works with a State containing an account."""
    from ethereum.forks.frontier.state import State, set_account
    from ethereum.forks.frontier.fork_types import Account, Address
    from ethereum_types.numeric import U256, Uint

    # Create a state and add an account
    state = State()
    address = Address(b'\x00' * 20)  # Zero address
    account = Account(
        nonce=Uint(1),
        balance=U256(1000),
        code=b"",
    )

    set_account(state, address, account)

    # Compute state root using rust_pyspec_utils
    rust_root = rust_pyspec_utils.state_root(state)

    # For now, we just check it returns a 32-byte hash
    assert isinstance(rust_root, bytes)
    assert len(rust_root) == 32


def test_rust_pyspec_utils_state_root_with_storage():
    """Test that rust_pyspec_utils.state_root works with a State containing storage."""
    from ethereum.forks.frontier.state import State, set_account, set_storage
    from ethereum.forks.frontier.fork_types import Account, Address
    from ethereum_types.numeric import U256, Uint
    from ethereum_types.bytes import Bytes32

    state = State()
    address = Address(b'\x00' * 19 + b'\x01')  # Address ending in 01
    account = Account(
        nonce=Uint(5),
        balance=U256(5000),
        code=b"",
    )

    set_account(state, address, account)

    set_storage(state, address, Bytes32(b'\x00' * 31 + b'\x01'), U256(100))
    set_storage(state, address, Bytes32(b'\x00' * 31 + b'\x02'), U256(200))

    rust_root = rust_pyspec_utils.state_root(state)

    # This test now passes correctly!
    from ethereum.forks.frontier.state import state_root as python_state_root
    python_root = python_state_root(state)

    assert isinstance(rust_root, bytes)
    assert len(rust_root) == 32
    assert rust_root == python_root


def test_rust_pyspec_utils_state_root_transaction_check():
    """Test that rust_pyspec_utils.state_root raises an error during a transaction."""
    import pytest
    from ethereum.forks.frontier.state import State, begin_transaction

    state = State()
    begin_transaction(state)

    with pytest.raises(AssertionError, match="Cannot compute state root during a transaction"):
        rust_pyspec_utils.state_root(state)
