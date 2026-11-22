
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract EvidenceAnchor {
    address public owner;
    address public recorder;

    struct Anchor {
        uint256 caseId;
        uint256 anchoredAt;
        bool exists;
    }

    mapping(bytes32 => Anchor) private anchors;

    event EvidenceAnchored(
        bytes32 indexed evidenceHash,
        uint256 indexed caseId,
        uint256 anchoredAt,
        address indexed recorder
    );

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner");
        _;
    }

    modifier onlyRecorder() {
        require(msg.sender == recorder, "Only recorder");
        _;
    }

    constructor() {
        owner = msg.sender;
        recorder = msg.sender;
    }

    function setRecorder(address newRecorder) external onlyOwner {
        require(newRecorder != address(0), "Invalid recorder");
        recorder = newRecorder;
    }

    function anchorEvidence(
        bytes32 evidenceHash,
        uint256 caseId
    ) external onlyRecorder {
        require(evidenceHash != bytes32(0), "Empty evidence hash");
        require(caseId > 0, "Invalid case ID");
        require(!anchors[evidenceHash].exists, "Already anchored");

        uint256 timestamp = block.timestamp;

        anchors[evidenceHash] = Anchor({
            caseId: caseId,
            anchoredAt: timestamp,
            exists: true
        });

        emit EvidenceAnchored(
            evidenceHash,
            caseId,
            timestamp,
            msg.sender
        );
    }

    function verifyEvidence(
        bytes32 evidenceHash
    )
        external
        view
        returns (bool exists, uint256 caseId, uint256 anchoredAt)
    {
        Anchor memory anchor = anchors[evidenceHash];

        return (
            anchor.exists,
            anchor.caseId,
            anchor.anchoredAt
        );
    }
}
