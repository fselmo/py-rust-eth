"""Comprehensive tests comparing rust_pyspec_utils with Python implementation"""

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


def test_compare_empty_state():
    """Compare empty state root between Rust and Python implementations."""
    from ethereum.forks.frontier.state import State
    from ethereum.forks.frontier.state import state_root as python_state_root

    state = State()

    rust_root = rust_pyspec_utils.state_root(state)
    python_root = python_state_root(state)

    print(f"Rust root:   {rust_root.hex()}")
    print(f"Python root: {python_root.hex()}")

    assert rust_root == python_root, f"Mismatch: Rust={rust_root.hex()}, Python={python_root.hex()}"


def test_compare_single_account_no_storage():
    """Compare state root with single account (no storage) between implementations."""
    from ethereum.forks.frontier.state import State, set_account
    from ethereum.forks.frontier.state import state_root as python_state_root
    from ethereum.forks.frontier.fork_types import Account, Address
    from ethereum_types.numeric import U256, Uint

    state = State()

    # Create an account with some balance
    address = Address(bytes.fromhex("1234567890123456789012345678901234567890"))
    account = Account(
        nonce=Uint(0),
        balance=U256(1000000000000000000),  # 1 ETH
        code=b"",
    )

    set_account(state, address, account)

    rust_root = rust_pyspec_utils.state_root(state)
    python_root = python_state_root(state)

    print(f"Rust root:   {rust_root.hex()}")
    print(f"Python root: {python_root.hex()}")

    assert rust_root == python_root, f"Mismatch: Rust={rust_root.hex()}, Python={python_root.hex()}"


def test_compare_account_with_code():
    """Compare state root with account having code."""
    from ethereum.forks.frontier.state import State, set_account
    from ethereum.forks.frontier.state import state_root as python_state_root
    from ethereum.forks.frontier.fork_types import Account, Address
    from ethereum_types.numeric import U256, Uint

    state = State()

    # Create an account with code
    address = Address(bytes.fromhex("2234567890123456789012345678901234567890"))
    account = Account(
        nonce=Uint(5),
        balance=U256(2000000000000000000),  # 2 ETH
        code=bytes.fromhex("6005600401"),  # Some bytecode
    )

    set_account(state, address, account)

    rust_root = rust_pyspec_utils.state_root(state)
    python_root = python_state_root(state)

    print(f"Rust root:   {rust_root.hex()}")
    print(f"Python root: {python_root.hex()}")

    assert rust_root == python_root, f"Mismatch: Rust={rust_root.hex()}, Python={python_root.hex()}"


def test_compare_account_with_storage():
    """Compare state root with account having storage."""
    from ethereum.forks.frontier.state import State, set_account, set_storage
    from ethereum.forks.frontier.state import state_root as python_state_root
    from ethereum.forks.frontier.fork_types import Account, Address
    from ethereum_types.numeric import U256, Uint
    from ethereum_types.bytes import Bytes32

    state = State()

    # Create an account
    address = Address(bytes.fromhex("3334567890123456789012345678901234567890"))
    account = Account(
        nonce=Uint(1),
        balance=U256(500000000000000000),  # 0.5 ETH
        code=b"",
    )

    set_account(state, address, account)

    # Add some storage
    set_storage(state, address, Bytes32(b'\x00' * 31 + b'\x01'), U256(100))
    set_storage(state, address, Bytes32(b'\x00' * 31 + b'\x02'), U256(200))
    set_storage(state, address, Bytes32(b'\x00' * 31 + b'\x03'), U256(300))

    rust_root = rust_pyspec_utils.state_root(state)
    python_root = python_state_root(state)

    print(f"Rust root:   {rust_root.hex()}")
    print(f"Python root: {python_root.hex()}")

    assert rust_root == python_root, f"Mismatch: Rust={rust_root.hex()}, Python={python_root.hex()}"


def test_compare_multiple_accounts():
    """Compare state root with multiple accounts."""
    from ethereum.forks.frontier.state import State, set_account
    from ethereum.forks.frontier.state import state_root as python_state_root
    from ethereum.forks.frontier.fork_types import Account, Address
    from ethereum_types.numeric import U256, Uint

    state = State()

    # Add multiple accounts
    accounts_data = [
        ("1111111111111111111111111111111111111111", 0, 1000, b""),
        ("2222222222222222222222222222222222222222", 1, 2000, bytes.fromhex("6005")),
        ("3333333333333333333333333333333333333333", 2, 3000, b""),
        ("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 10, 5000, bytes.fromhex("600160025503")),
    ]

    for addr_hex, nonce, balance, code in accounts_data:
        address = Address(bytes.fromhex(addr_hex))
        account = Account(
            nonce=Uint(nonce),
            balance=U256(balance),
            code=code,
        )
        set_account(state, address, account)

    rust_root = rust_pyspec_utils.state_root(state)
    python_root = python_state_root(state)

    print(f"Rust root:   {rust_root.hex()}")
    print(f"Python root: {python_root.hex()}")

    assert rust_root == python_root, f"Mismatch: Rust={rust_root.hex()}, Python={python_root.hex()}"