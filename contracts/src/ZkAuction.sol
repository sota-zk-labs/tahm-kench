// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721.sol";

contract ZkAuction {
    IERC20 public immutable lockToken; // The token to be locked

    // data for verifying batch inclusion
    bytes32 public constant ELF_COMMITMENT = 0xe2a6abcfe3391a99ab84db41d4d1f5aead0ad92930932f20ff57c5d9a2fcc203;
    error InvalidElf(bytes32 submittedElf);
    address public constant ALIGNED_SERVICE_MANAGER = 0x58F280BeBE9B34c9939C3C39e0890C81f163B623;
    address public constant ALIGNED_PAYMENT_SERVICE_ADDR = 0x815aeCA64a974297942D2Bbf034ABEe22a38A003;

    struct Auction {
        address owner; // Owner of the auction
        bytes ownerPublicKey; // Owner's public key
        Asset asset; // Asset being auctioned
        Item item; // The item to be auction
        Bid[] bids; // Array of bids placed on the auction
        Winner winner; // Winner of the auction
        uint256 depositPrice; // Deposit price when bider start bid
        mapping(address => bool) hasDeposited; // To track if a bidder has deposited
        uint256 endTime; // Time when the bid phase end
        bool ended; // Status of the auction
    }

    struct Bid {
        address bidder; // Address of the bidder
        bytes encryptedPrice; // Encrypted price submitted of bidder
    }

    struct Asset {
        string name; // Name of the asset
        string description; // Description of the asset
    }

    struct Item {
        IERC721 nftContract; // The nft to be auction
        uint256 tokenId; // Id nft
    }

    struct Winner {
        address winner; // Address of the winner
        uint256 price; // Price submitted of winner
    }

    // Mapping to store auctions by their ID
    mapping(uint256 => Auction) public auctions;
    uint256 public auctionCount; // Counter for auctions

    // Events
    event AuctionCreated(uint256 indexed auctionId, address indexed owner);
    event NewBid(
        uint256 indexed auctionId,
        address indexed bidder,
        bytes encryptedPrice
    );
    event AuctionEnded(
        uint256 indexed auctionId,
        address indexed winner,
        uint256 price
    );

    constructor(address _lockToken) {
        lockToken = IERC20(_lockToken);
    }

    // Function to create a new auction
    /**
     * @notice Creates a new auction with specific parameters.
     * @dev Initializes a new auction.
     */
    function createAuction(
        bytes memory _ownerPublicKey,
        address _nftContract,
        uint256 _tokenId,
        string memory _assetName,
        string memory _assetDescription,
        uint256 _depositPrice,
        uint256 _duration
    ) public {
        require(_depositPrice > 0, "Deposit price must be greater than zero");
        require(_duration > 0, "Duration must be greater than zero");

        IERC721 nftContract = IERC721(_nftContract);
        require(
            nftContract.ownerOf(_tokenId) == msg.sender,
            "You must own the NFT to auction it"
        );
        require(
            nftContract.getApproved(_tokenId) == address(this),
            "You need approve the NFT to contract"
        );

        // Create auction
        auctionCount++;
        Auction storage newAuction = auctions[auctionCount];

        newAuction.owner = msg.sender;
        newAuction.ownerPublicKey = _ownerPublicKey;
        newAuction.item = Item(nftContract, _tokenId);
        newAuction.asset = Asset(_assetName, _assetDescription);
        newAuction.depositPrice = _depositPrice;
        newAuction.endTime = block.timestamp + _duration; // Set auction end time
        newAuction.ended = false;

        // Deposit item
        nftContract.transferFrom(msg.sender, address(this), _tokenId); // Transfer NFT to contract
        require(
            nftContract.ownerOf(_tokenId) == address(this),
            "Auction contract must own the NFT."
        );
        emit AuctionCreated(auctionCount, msg.sender);
    }

    /**
     * @notice Allows users to place bids.
     * @dev Bids are encrypted for ZK-based auctions.
     */
    function new_bid(uint256 auctionId, bytes memory _encryptedPrice) public {
        Auction storage auction = auctions[auctionId];
        require(!auction.ended, "Auction has ended");
        require(block.timestamp < auction.endTime, "Auction has expired");
        require(!auction.hasDeposited[msg.sender], "Already deposited");
        require(
            lockToken.allowance(msg.sender, address(this)) ==
            auction.depositPrice,
            "You need approve token deposit to contract"
        );
        // Update the state to indicate that the user has deposited
        auction.hasDeposited[msg.sender] = true;
        // Bid
        auction.bids.push(
            Bid({bidder: msg.sender, encryptedPrice: _encryptedPrice})
        );

        lockToken.transferFrom(msg.sender, address(this), auction.depositPrice);
        emit NewBid(auctionId, msg.sender, _encryptedPrice);
    }

    /**
     * @notice Gets list bidders after the bid phase end
     * @dev Uses auctionId to get list bidders.
     */
    function getBids(uint256 auctionId) view public returns (Bid[] memory) {
        require(
            block.timestamp >= auctions[auctionId].endTime,
            "Auction has not ended yet"
        );
        require(!auctions[auctionId].ended, "Auction has ended");
        return auctions[auctionId].bids;
    }


    function withdraw(uint256 auctionId) public {
        Auction storage auction = auctions[auctionId];

        /**
         * @notice Reveals the winner after the auction ends.
     *
     * This function reveals the highest valid bid by a ZK-proof and transfers ownership of the NFT to the winning bidder.
     *
     * Parameters:
     *     _auctionId (uint256): The ID of the auction being ended
     *     winner (Winner memory): Information about the winner, including their address and price submitted in encrypted form
     *     inputHash (bytes32): A hash used for verification purposes only
     *     verifiedProofData (bytes) - bytes: Data provided by a ZK-proving system to verify that this bid was made during the auction phase.
     *
     * Requirements:
     * 1. The current block timestamp must be greater than or equal to the end time of the auction (_endTime).
     * 2. The `ended` status flag for the associated Auction contract instance is set to false, indicating it has not ended yet
     * 3. A verification data hash and a proof are submitted in _verifiedProofData that matches the batch inclusion information with ELF COMMITMENT.
     *
     * Effects:
     *   - Set winner of auction
     *   - Transfer NFT from this smart contract address back into possession of winning bidder's address
     *   - Send difference between total deposit price for entire bid period and actual highest valid encryptedPrice to user in token form, if any exists (otherwise zero is sent).
     *
     */
        /**
         * @notice Reveals the winner after the auction ends.
     *
     * This function reveals the highest valid bid by a ZK-proof and transfers ownership of the NFT to the winning bidder.
     *
     * Parameters:
     *     _auctionId (uint256): The ID of the auction being ended
     *     winner (Winner memory): Information about the winner, including their address and price submitted in encrypted form
     *     inputHash (bytes32): A hash used for verification purposes only
     *     verifiedProofData (bytes) - bytes: Data provided by a ZK-proving system to verify that this bid was made during the auction phase.
     *
     * Requirements:
     * 1. The current block timestamp must be greater than or equal to the end time of the auction (_endTime).
     * 2. The `ended` status flag for the associated Auction contract instance is set to false, indicating it has not ended yet
     * 3. A verification data hash and a proof are submitted in _verifiedProofData that matches the batch inclusion information with ELF COMMITMENT.
     *
     * Effects:
     *   - Set winner of auction
     *   - Transfer NFT from this smart contract address back into possession of winning bidder's address
     *   - Send difference between total deposit price for entire bid period and actual highest valid encryptedPrice to user in token form, if any exists (otherwise zero is sent).
     *
     */     require(auction.hasDeposited[msg.sender], "No tokens to withdraw");
        require(auction.ended, "Tokens are still locked");
        // Transfer tokens from this contract to the user
        lockToken.transfer(msg.sender, auction.depositPrice);
    }

    /**
     * @notice Reveals the winner after the auction ends.
     * @dev Uses a ZK-proof to reveal the highest valid bid.
     */
    function revealWinner(
        uint256 auctionId,
        Winner memory winner,
        bytes memory verifiedProofData
    ) public {
        require(
            block.timestamp >= auctions[auctionId].endTime,
            "Auction has not ended yet"
        );
        require(!auctions[auctionId].ended, "Auction has ended");
        verifyProof(winner, auctionId, verifiedProofData);
        // Set winner
        Auction storage auction = auctions[auctionId];
        require(
            winner.price <= auction.depositPrice,
            "Winner has more bid price than deposit price"
        );
        auction.winner = winner;
        // Send item
        auction.item.nftContract.transferFrom(
            address(this),
            auction.winner.winner,
            auction.item.tokenId
        );
        // Refund token
        if (auction.depositPrice > auction.winner.price) {
            lockToken.transfer(
                auction.winner.winner,
                auction.depositPrice - auction.winner.price
            );
        }
        // Withdraw token
        lockToken.transfer(msg.sender, auction.winner.price);
        // Set status auction
        auction.ended = true;

        emit AuctionEnded(
            auctionId,
            auction.winner.winner,
            auction.winner.price
        );
    }

    function verifyProof(
        Winner memory winner,
        uint256 auctionId,
        bytes memory verifiedProofData
    ) internal view {
        (
            bytes memory publicInput,
            bytes32 proofCommitment,
            bytes32 pubInputCommitment,
            bytes32 provingSystemAuxDataCommitment,
            bytes20 proofGeneratorAddr,
            bytes32 batchMerkleRoot,
            bytes memory merkleProof,
            uint256 verificationDataBatchIndex
        ) = abi.decode(verifiedProofData, (bytes, bytes32, bytes32, bytes32, bytes20, bytes32, bytes, uint256));
        if (ELF_COMMITMENT != provingSystemAuxDataCommitment) {
            revert InvalidElf(provingSystemAuxDataCommitment);
        }
        require(
            address(proofGeneratorAddr) == msg.sender,
            "proofGeneratorAddr does not match"
        );
        require(
            pubInputCommitment == keccak256(publicInput),
            "Invalid public input"
        );

        (bytes32 auctionHash, address winner_addr) = abi.decode(
            publicInput,
            (bytes32, address)
        );

        require(winner_addr == winner.winner, "Winner in proof does not match");
        require(calculateAuctionHash(auctionId) == auctionHash, "Auction hash does not match");

        (
            bool callWasSuccessful,
            bytes memory proofIsIncluded
        ) = ALIGNED_SERVICE_MANAGER.staticcall(
            abi.encodeWithSignature(
                "verifyBatchInclusion(bytes32,bytes32,bytes32,bytes20,bytes32,bytes,uint256,address)",
                proofCommitment,
                pubInputCommitment,
                provingSystemAuxDataCommitment,
                proofGeneratorAddr,
                batchMerkleRoot,
                merkleProof,
                verificationDataBatchIndex,
                ALIGNED_PAYMENT_SERVICE_ADDR
            )
        );

        require(callWasSuccessful, "static_call failed");

        bool proofIsIncludedBool = abi.decode(proofIsIncluded, (bool));
        require(proofIsIncludedBool, "proof not included in batch");
    }

    function calculateAuctionHash(uint256 auctionId) view internal returns (bytes32) {
        Bid[] memory bids = auctions[auctionId].bids;
        bytes memory hashInput = abi.encodePacked(auctionId);
        for (uint256 i = 0; i < bids.length; ++i) {
            hashInput = abi.encodePacked(hashInput, bids[i].bidder, bids[i].encryptedPrice);
        }
        return keccak256(hashInput);
    }
}
