
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {EvidenceAnchor} from "../src/EvidenceAnchor.sol";

contract EvidenceAnchorTest is Test {
    EvidenceAnchor private anchor;

    address private owner = address(this);
    address private backendRecorder = address(0xBEEF);
    address private unauthorizedUser = address(0xBAD);

    bytes32 private evidenceHash =
        0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824;

    uint256 private caseId = 42;

    function setUp() public {
        anchor = new EvidenceAnchor();
        anchor.setRecorder(backendRecorder);
    }

    function testAuthorizedRecorderCanAnchorEvidence() public {
        vm.prank(backendRecorder);

        anchor.anchorEvidence(evidenceHash, caseId);

        (bool exists, uint256 storedCaseId, uint256 anchoredAt) =
            anchor.verifyEvidence(evidenceHash);

        assertTrue(exists);
        assertEq(storedCaseId, caseId);
        assertEq(anchoredAt, block.timestamp);
    }

    function testUnauthorizedUserCannotAnchorEvidence() public {
        vm.prank(unauthorizedUser);

        vm.expectRevert("Only recorder");
        anchor.anchorEvidence(evidenceHash, caseId);
    }

    function testEvidenceCannotBeAnchoredTwice() public {
        vm.prank(backendRecorder);
        anchor.anchorEvidence(evidenceHash, caseId);

        vm.prank(backendRecorder);
        vm.expectRevert("Already anchored");
        anchor.anchorEvidence(evidenceHash, caseId);
    }

    function testCannotAnchorEmptyHash() public {
        vm.prank(backendRecorder);

        vm.expectRevert("Empty evidence hash");
        anchor.anchorEvidence(bytes32(0), caseId);
    }

    function testCannotAnchorInvalidCaseId() public {
        vm.prank(backendRecorder);

        vm.expectRevert("Invalid case ID");
        anchor.anchorEvidence(evidenceHash, 0);
    }

    function testUnanchoredEvidenceReturnsFalse() public view {
        (bool exists, uint256 storedCaseId, uint256 anchoredAt) =
            anchor.verifyEvidence(evidenceHash);

        assertFalse(exists);
        assertEq(storedCaseId, 0);
        assertEq(anchoredAt, 0);
    }

    function testOnlyOwnerCanChangeRecorder() public {
        vm.prank(unauthorizedUser);

        vm.expectRevert("Only owner");
        anchor.setRecorder(address(0xCAFE));
    }

    function testOwnerCanChangeRecorder() public {
        address newRecorder = address(0xCAFE);

        anchor.setRecorder(newRecorder);

        assertEq(anchor.recorder(), newRecorder);
    }
}
