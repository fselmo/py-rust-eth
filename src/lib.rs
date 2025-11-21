use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use alloy_primitives::{Address, B256, U256};
use alloy_trie::{HashBuilder, Nibbles};
use alloy_rlp::Encodable;

/// Compute the storage root for a given address
fn compute_storage_root(storage_tries: &Bound<'_, PyAny>, address_bytes: &Bound<'_, PyBytes>) -> PyResult<B256> {
    use alloy_primitives::keccak256;

    // Check if this address has a storage trie
    let storage_dict = storage_tries.downcast::<PyDict>()?;

    // Empty storage root (used when account has no storage)
    let empty_root = B256::from([
        0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6,
        0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
        0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0,
        0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21
    ]);

    // Check if this address has storage
    if !storage_dict.contains(address_bytes)? {
        return Ok(empty_root);
    }

    // Get the storage trie for this address
    let storage_trie = storage_dict.get_item(address_bytes)?;
    if let Some(trie) = storage_trie {
        // Get the _data from the storage trie
        let trie_data = trie.getattr("_data")?;
        let trie_dict = trie_data.downcast::<PyDict>()?;

        // If the storage trie is empty, return empty root
        if trie_dict.is_empty() {
            return Ok(empty_root);
        }

        // Create a HashBuilder for computing the storage root
        let mut hash_builder = HashBuilder::default();

        // Collect and sort storage entries
        let mut storage_entries: Vec<(B256, U256)> = Vec::new();

        // Iterate through storage slots to collect them
        for (key_bytes, value_obj) in trie_dict.iter() {
            // Skip None values
            if value_obj.is_none() {
                continue;
            }

            // Extract key (slot) bytes
            let key_py_bytes = key_bytes.downcast::<PyBytes>()?;
            let slot_bytes = key_py_bytes.as_bytes();

            if slot_bytes.len() != 32 {
                continue; // Skip invalid slots
            }

            // Extract value (U256)
            let value_bytes: Vec<u8> = value_obj.call_method0("to_be_bytes32")?.extract()?;
            let value = U256::from_be_slice(&value_bytes);

            // Skip zero values (they should be deleted from storage)
            if value.is_zero() {
                continue;
            }

            let slot_b256 = B256::from_slice(slot_bytes);
            storage_entries.push((slot_b256, value));
        }

        // Sort by hashed keys (this is crucial for the trie)
        storage_entries.sort_by_key(|(slot, _)| keccak256(*slot));

        // Now add them to the hash builder in sorted order
        for (slot, value) in storage_entries {
            // For secured tries, we need to hash the slot key first
            let hashed_slot = keccak256(slot);
            let key_nibbles = Nibbles::unpack(hashed_slot);

            // RLP encode the value
            let mut value_rlp = Vec::new();
            value.encode(&mut value_rlp);

            // Add to hash builder
            hash_builder.add_leaf(key_nibbles, &value_rlp);
        }

        // Compute and return the storage root
        Ok(hash_builder.root())
    } else {
        Ok(empty_root)
    }
}

/// rust-pyspec-utils: Rust acceleration library for Ethereum execution and consensus specs
///
/// This module provides high-performance implementations of computationally intensive
/// Ethereum operations, designed to work seamlessly with the Python execution-specs
/// and potentially consensus-specs projects.
#[pymodule]
fn rust_pyspec_utils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(state_root, m)?)?;
    Ok(())
}

/// Compute the state root for a given Python State object
///
/// This function is designed to be a drop-in replacement for the Python
/// `state_root(state: State) -> Root` function in ethereum.forks.{fork}.state
///
/// Parameters
/// ----------
/// state : State
///     The Python State object from ethereum.forks.{fork}.state
///
/// Returns
/// -------
/// root : bytes
///     The 32-byte state root hash
#[pyfunction]
fn state_root(state: &Bound<'_, PyAny>) -> PyResult<Py<PyBytes>> {
    // Extract the _main_trie from the State object
    let main_trie = state.getattr("_main_trie")?;

    // Extract the _storage_tries from the State object
    let storage_tries = state.getattr("_storage_tries")?;

    // Check that there are no active snapshots (transactions)
    let snapshots = state.getattr("_snapshots")?;
    let snapshots_len: usize = snapshots.len()?;
    if snapshots_len > 0 {
        return Err(pyo3::exceptions::PyAssertionError::new_err(
            "Cannot compute state root during a transaction"
        ));
    }

    let py = state.py();

    // Try to get the data from main_trie
    // Note: The Python trie structure has a _data attribute that holds the accounts
    let trie_data = main_trie.getattr("_data")?;
    let trie_dict = trie_data.downcast::<PyDict>()?;

    // If the trie is empty, return the empty trie root
    if trie_dict.is_empty() {
        // Empty state root (Keccak256 of RLP empty string)
        // This is the correct empty trie root: 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421
        let empty_root = [
            0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6,
            0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
            0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0,
            0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21
        ];
        return Ok(PyBytes::new_bound(py, &empty_root).into());
    }

    // Create a HashBuilder for computing the Merkle Patricia Trie root
    let mut hash_builder = HashBuilder::default();

    // Collect all accounts first
    let mut accounts: Vec<(Address, u64, U256, Vec<u8>, B256)> = Vec::new();

    // Iterate through accounts in the main trie
    for (address_bytes, account_obj) in trie_dict.iter() {
        // Skip None accounts
        if account_obj.is_none() {
            continue;
        }

        // Extract address bytes
let address_py_bytes = address_bytes.downcast::<PyBytes>()?;
        let addr_bytes = address_py_bytes.as_bytes();

        if addr_bytes.len() != 20 {
            continue; // Skip invalid addresses
        }

        // Convert to Address
        let mut addr_array = [0u8; 20];
        addr_array.copy_from_slice(addr_bytes);
        let address = Address::from(addr_array);

        // Extract account info
        let account = account_obj;
        let nonce: u64 = account.getattr("nonce")?.extract()?;
        let balance_obj = account.getattr("balance")?;

        // Handle balance extraction (it's a U256 type from ethereum_types)
        // Convert to bytes and then back to U256
        let balance_bytes: Vec<u8> = balance_obj.call_method0("to_be_bytes32")?.extract()?;
        let balance = U256::from_be_slice(&balance_bytes);

        // Get code (bytecode)
        let code: Vec<u8> = account.getattr("code")?.extract()?;

        // Compute storage root for this address
        let storage_root = compute_storage_root(&storage_tries, &address_py_bytes)?;

        // Collect the account data
        accounts.push((address, nonce, balance, code, storage_root));
    }

    // Sort accounts by hashed address (crucial for trie)
    use alloy_primitives::keccak256;
    accounts.sort_by_key(|(addr, _, _, _, _)| keccak256(*addr));

    // Now process accounts in sorted order
    for (address, nonce, balance, code, storage_root) in accounts {
        // Compute code hash (Keccak256 of code)
        let code_hash = if code.is_empty() {
            // Empty code hash
            B256::from([
                0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c,
                0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
                0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b,
                0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70
            ])
        } else {
            // Calculate actual code hash
            keccak256(&code)
        };

        // Create account RLP encoding as a list
        // Account encoding: [nonce, balance, storage_root, code_hash]
        use alloy_rlp::RlpEncodable;

        // Create a temporary struct just for encoding
        #[derive(RlpEncodable)]
        struct TempAccount<'a> {
            nonce: u64,
            balance: U256,
            storage_root: &'a [u8; 32],
            code_hash: &'a [u8; 32],
        }

        let temp_account = TempAccount {
            nonce,
            balance,
            storage_root: storage_root.as_ref(),
            code_hash: code_hash.as_ref(),
        };

        let mut account_rlp = Vec::new();
        temp_account.encode(&mut account_rlp);

        // For secured tries, we need to hash the address key first
        let hashed_address = keccak256(address);
        let address_nibbles = Nibbles::unpack(hashed_address);

        // Add to hash builder
        hash_builder.add_leaf(address_nibbles, &account_rlp);
    }

    // Compute the root hash
    let root = hash_builder.root();

    // Convert B256 to bytes and return
    Ok(PyBytes::new_bound(py, root.as_slice()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_imports() {
        // Basic test to ensure the module compiles
        assert!(true);
    }
}