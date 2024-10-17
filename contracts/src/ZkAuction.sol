// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

contract ZkAuction {
    struct Auction {
        address owner; // Owner of the auction
        bytes ownerPublicKey; // Owner's public key
        Asset asset; // Asset being auctioned
        Bid[] bids; // Array of bids placed on the auction
        Winner winner; // Winner of the auction
        uint256 depositPrice; // Deposit price when bider start bid
        mapping(address => bool) hasDeposited; // To track if a bidder has deposited
        uint256 endTime; // Time when the auction ends
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

    struct Winner {
        address winner; // Address of the winner
        bytes encryptedPrice; // Encrypted price submitted of winner
    }

    // Mapping to store auctions by their ID
    mapping(uint256 => Auction) public auctions;
    uint256 public auctionCount; // Counter for auctions

    // Events
    event AuctionCreated(uint256 indexed auctionId, address indexed owner);
    event NewBid(uint256 indexed auctionId, address indexed bidder, bytes encryptedPrice);
    event AuctionEnded(uint256 indexed auctionId, address indexed winner, bytes encryptedPrice);

    // Function to create a new auction
    /**
     * @notice Creates a new auction with specific parameters.
     * @dev Initializes a new auction.
     */
    function createAuction(bytes memory _ownerPublicKey, string memory _assetName, string memory _assetDescription, uint256 _depositPrice, uint256 _duration) public {
        // Logic for creating an auction
        auctionCount ++;
        Auction storage newAuction = auctions[auctionCount];

        newAuction.owner = msg.sender;
        newAuction.ownerPublicKey = _ownerPublicKey;
        newAuction.asset = Asset(_assetName, _assetDescription);
        newAuction.depositPrice = _depositPrice;
        newAuction.endTime = block.timestamp + _duration; // Set auction end time
        newAuction.ended = false;

        emit AuctionCreated(auctionCount, msg.sender);
    }
}
