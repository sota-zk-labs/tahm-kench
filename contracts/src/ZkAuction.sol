// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract ZkAuction {
    IERC20 public immutable lockToken; // The token to be locked

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

    constructor(address _lockToken) {
        lockToken = IERC20(_lockToken);
    }

    // Function to create a new auction
    /**
     * @notice Creates a new auction with specific parameters.
     * @dev Initializes a new auction.
     */
    function createAuction(bytes memory _ownerPublicKey, string memory _assetName, string memory _assetDescription, uint256 _depositPrice, uint256 _duration) public {
        require(_depositPrice > 0, "Deposit price must be greater than zero");
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

    function deposit(uint256 auction_id) public {
        Auction storage auction = auctions[auction_id];
        require(!auction.hasDeposited[msg.sender], "Already deposited");
        require(!auction.ended, "Auction ended");
        // Transfer tokens from the user to this contract
        lockToken.transferFrom(msg.sender, address(this), auction.depositPrice);
        // Update the state to indicate that the user has deposited
        auction.hasDeposited[msg.sender] = true;
    }

    function unlock(uint256 auctionId) public {
        Auction storage auction = auctions[auctionId];
        require(auction.hasDeposited[msg.sender], "No tokens to unlock");
        require(auction.ended, "Tokens are still locked");
        // Transfer tokens from this contract to the user
        lockToken.transfer(msg.sender, auction.depositPrice);
    }

    /**
     * @notice Allows users to place bids.
     * @dev Bids are encrypted for ZK-based auctions.
     */
    function bid(uint256 auctionId, bytes memory _encryptedPrice) public {
        // Logic for placing a bid
        Auction storage auction = auctions[auctionId];
        require(!auction.ended, "Auction has ended.");
        require(block.timestamp < auction.endTime, "Auction has expired.");
        require(auction.hasDeposited[msg.sender], "You must deposit before bidding.");

        auction.bids.push(Bid({
            bidder: msg.sender,
            encryptedPrice: _encryptedPrice
        }));

        emit NewBid(auctionId, msg.sender, _encryptedPrice);
    }

}
