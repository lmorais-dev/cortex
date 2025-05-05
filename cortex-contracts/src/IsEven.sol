// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {IIsEven} from "./IIsEven.sol";
import {ImageID} from "./ImageID.sol";

contract IsEven is IIsEven {
    IRiscZeroVerifier private verifier;
    bytes32 private constant imageId = ImageID.CORTEX_ZK_ISEVEN_ID;

    mapping(address => uint128) private evenNumber;

    constructor(IRiscZeroVerifier _verifier) {
        verifier = IRiscZeroVerifier(_verifier);
    }

    function set(uint128 number, bytes calldata seal) external {
        bytes memory journal = abi.encode(number);
        verifier.verify(seal, imageId, sha256(journal));
        evenNumber[msg.sender] = number;
    }

    function get() external view override returns (uint128) {
        return evenNumber[msg.sender];
    }
}