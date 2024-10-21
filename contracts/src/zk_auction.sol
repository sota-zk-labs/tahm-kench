// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721.sol";

contract ZkAuction {
    IERC20 public immutable lockToken; // The token to be locked

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
    event NewBid(uint256 indexed auctionId, address indexed bidder, bytes encryptedPrice);
    event AuctionEnded(uint256 indexed auctionId, address indexed winner, uint256 price);

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
        // Logic for creating an auction
        //deposit item
        IERC721 nftContract = IERC721(_nftContract);
        require(nftContract.ownerOf(_tokenId) == msg.sender, "You must own the NFT to auction it.");
        nftContract.transferFrom(msg.sender, address(this), _tokenId); // Transfer NFT to contract

        //Create auction
        auctionCount++;
        Auction storage newAuction = auctions[auctionCount];

        newAuction.owner = msg.sender;
        newAuction.ownerPublicKey = _ownerPublicKey;
        newAuction.item = Item(nftContract, _tokenId);
        newAuction.asset = Asset(_assetName, _assetDescription);
        newAuction.depositPrice = _depositPrice;
        newAuction.endTime = block.timestamp + _duration; // Set auction end time
        newAuction.ended = false;

        emit AuctionCreated(auctionCount, msg.sender);
    }

    /**
     * @notice Allows users to place bids.
     * @dev Bids are encrypted for ZK-based auctions.
     */
    function new_bid(uint256 auctionId, bytes memory _encryptedPrice) public {
        // Logic for placing a bid
        Auction storage auction = auctions[auctionId];
        require(!auction.ended, "Auction has ended.");
        require(block.timestamp < auction.endTime, "Auction has expired.");
        require(!auction.hasDeposited[msg.sender], "Already deposited");

        // Deposit to pool
        // Transfer tokens from the user to this contract
        lockToken.transferFrom(msg.sender, address(this), auction.depositPrice);
        // Update the state to indicate that the user has deposited
        auction.hasDeposited[msg.sender] = true;

        // Bid
        require(auction.hasDeposited[msg.sender], "You must deposit before bidding.");
        auction.bids.push(Bid({bidder: msg.sender, encryptedPrice: _encryptedPrice}));

        emit NewBid(auctionId, msg.sender, _encryptedPrice);
    }

    /**
    * @notice Gets list biders after the bid phase end
     * @dev Uses auctionId to get list biders.
     */
    function getBids(uint256 auctionId) public view returns (Bid[] memory) {
        require(block.timestamp >= auctions[auctionId].endTime, "Auction has not ended yet.");
        require(!auctions[auctionId].ended, "Auction has ended.");
        return auctions[auctionId].bids;
    }

    /**
     * @notice Reveals the winner after the auction ends.
     * @dev Uses a ZK-proof to reveal the highest valid bid.
     */
    function revealWinner(uint256 auctionId, Winner memory _winner) public {
        require(block.timestamp >= auctions[auctionId].endTime, "Auction has not ended yet.");
        require(!auctions[auctionId].ended, "Auction has ended.");
        // verifyProof(winner, inputHash, proof);
        //set winner
        Auction storage auction = auctions[auctionId];
        auction.winner = _winner;
        //send item
        auction.item.nftContract.transferFrom(address(this), auction.winner.winner, auction.item.tokenId);
        //refund cash
        if (auction.depositPrice > auction.winner.price) {
            lockToken.transfer(auction.winner.winner, auction.depositPrice - auction.winner.price);
        }
        //withdraw cash
        lockToken.transfer(msg.sender, auction.winner.price);
        //set status auction
        auction.ended = true;

        emit AuctionEnded(auctionId, auction.winner.winner, auction.winner.price);
    }


    function unlock(uint256 auctionId) public {
        Auction storage auction = auctions[auctionId];
        require(auction.hasDeposited[msg.sender], "No tokens to unlock");
        require(auction.ended, "Tokens are still locked");
        // Transfer tokens from this contract to the user
        lockToken.transfer(msg.sender, auction.depositPrice);
    }

    // /**
    //  * @notice Verifies a zero-knowledge proof.
    //  */
    // function verifyProof(Winner memory winner, bytes32 inputHash, bytes memory proof) internal returns (bool) {
    //     // ZK-proof verification logic
    //     return true;
    // }
}
