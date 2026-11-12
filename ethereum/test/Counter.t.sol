// SPDX-License-Identifier: MIT
pragma solidity 0.8.38;

import {Test} from "forge-std/Test.sol";
import {Counter as SolidityCounter} from "../src/Counter.sol";

/// @notice The surface both compilers expose for this counter.
interface ICounter {
    function count() external view returns (uint256);
    function increment() external;
    function set(uint256 newCount) external;
}

/// @dev Shared checks. Each concrete test only decides which compiler's
///      bytecode to deploy. Deploying is `new`: Forge compiles the source,
///      puts the code in a fresh contract account, and returns its address.
abstract contract CounterSuite is Test {
    function _deploy() internal virtual returns (ICounter);

    function test_deployReadsZero() public {
        ICounter counter = _deploy();
        assertEq(counter.count(), 0);
    }

    function test_increment() public {
        ICounter counter = _deploy();
        counter.increment();
        assertEq(counter.count(), 1);
        counter.increment();
        assertEq(counter.count(), 2);
    }

    function test_set() public {
        ICounter counter = _deploy();
        counter.set(42);
        assertEq(counter.count(), 42);
    }
}

contract SolidityCounterTest is CounterSuite {
    function _deploy() internal override returns (ICounter) {
        return ICounter(address(new SolidityCounter()));
    }
}

contract VyperCounterTest is CounterSuite {
    function _deploy() internal override returns (ICounter) {
        // Deploy the compiled Vyper artifact. Importing Counter.vy from this
        // Solidity file makes Foundry treat the Vyper version pragma as a solc
        // constraint, which cannot sit next to pragma 0.8.38.
        return ICounter(deployCode("Counter.vy"));
    }
}
