// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

interface IIsEven {
    function set(uint128 number, bytes calldata seal) external;
    function get() external view returns (uint128);
}
