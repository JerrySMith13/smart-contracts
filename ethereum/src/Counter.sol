// SPDX-License-Identifier: MIT
pragma solidity 0.8.38;

/// @title Counter
/// @notice A counter kept in this contract's own storage.
/// @dev `count` occupies storage slot 0. An externally owned account changes it
///      by sending a transaction that calls `increment` or `set`. Reading
///      `count()` is a call: it does not create a transaction and does not
///      write storage.
contract Counter {
    uint256 public count;

    /// @notice Add one to the stored count.
    function increment() external {
        count += 1;
    }

    /// @notice Replace the stored count.
    function set(uint256 newCount) external {
        count = newCount;
    }
}
