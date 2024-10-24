// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";

contract ZkAuction is IERC721Receiver {
    using SafeERC20 for IERC20;

    IERC20 public immutable lockToken; // The token to be locked

    struct Auction {
        address owner; // Owner of the auction
        bytes ownerPublicKey; // Owner's public key
        Asset asset; // Asset being auctioned
        Bid[] bids; // Array of bids placed on the auction
        Winner winner; // Winner of the auction
        uint256 depositPrice; // Deposit price when bidder start bid
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
        address nftContract; // The nft to be auction
        uint256 tokenId; // Id nft
    }

    struct Winner {
        address winner; // Address of the winner
        uint256 price; // Price submitted of winner
    }

    uint256 public auctionCount; // Counter for auctions

    // Mapping to store auctions by their ID
    mapping(uint256 => Auction) public auctions;
    // Mapping from owner address to their list of auctions
    mapping(address => Auction[]) public auctionsByOwner;
    // Mapping to track deposits for each auction
    mapping(uint256 => mapping(address => bool)) public hasDeposited;

    // Events
    event AuctionCreated(uint256 indexed auctionId, address indexed owner);
    event NewBid(uint256 indexed auctionId, address indexed bidder, bytes encryptedPrice);
    event AuctionEnded(uint256 indexed auctionId, address indexed winner, uint256 price);

    constructor(address _lockToken) {
        lockToken = IERC20(_lockToken);
    }

    modifier onlyOwner(uint256 auctionId) {
        Auction storage auction = auctions[auctionId];
        require(msg.sender == auction.owner, "You are not the owner");
        _;
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
        require(nftContract.ownerOf(_tokenId) == msg.sender, "You must own the NFT to auction it");
        require(nftContract.getApproved(_tokenId) == address(this), "You need approve the NFT to contract");

        // Create auction
        auctionCount++;
        Auction storage newAuction = auctions[auctionCount];

        newAuction.owner = msg.sender;
        newAuction.ownerPublicKey = _ownerPublicKey;
        newAuction.asset = Asset(_assetName, _assetDescription, _nftContract, _tokenId);
        newAuction.depositPrice = _depositPrice;
        newAuction.endTime = block.timestamp + _duration; // Set auction end time
        newAuction.ended = false;

        auctionCount++;
        auctionsByOwner[msg.sender].push(newAuction);

        // Deposit nft
        nftContract.safeTransferFrom(msg.sender, address(this), _tokenId); // Transfer NFT to contract
        emit AuctionCreated(auctionCount, msg.sender);
    }

    // Get auctions for an owner
    function getAuctionsByOwner(address owner) public view returns (Auction[] memory) {
        return auctionsByOwner[owner];
    }

    /**
     * @notice Allows users to place bids.
     * @dev Bids are encrypted for ZK-based auctions.
     */
    function placeBid(uint256 auctionId, bytes memory _encryptedPrice) public {
        Auction storage auction = auctions[auctionId];
        require(!auction.ended, "Auction has ended");
        require(block.timestamp < auction.endTime, "Auction has expired");
        require(!auction.hasDeposited[msg.sender], "Already deposited");
        require(
            lockToken.allowance(msg.sender, address(this)) == auction.depositPrice,
            "You need approve token deposit to contract"
        );
        // Update the state to indicate that the user has deposited
        hasDeposited[auctionId][msg.sender] = true;
        // Bid
        auction.bids.push(Bid({bidder: msg.sender, encryptedPrice: _encryptedPrice}));

        lockToken.safeTransferFrom(msg.sender, address(this), auction.depositPrice);
        emit NewBid(auctionId, msg.sender, _encryptedPrice);
    }

    /**
     * @notice Gets list bidders after the bid phase end
     * @dev Uses auctionId to get list bidders.
     */
    function getBids(uint256 auctionId) public view returns (Bid[] memory) {
        require(block.timestamp >= auctions[auctionId].endTime, "Auction has not ended yet");
        require(!auctions[auctionId].ended, "Auction has ended");
        return auctions[auctionId].bids;
    }

    /**
     * @notice Reveals the winner after the auction ends.
     * @dev Uses a ZK-proof to reveal the highest valid bid.
     */
    function finalizeAuction(uint256 auctionId, Winner memory _winner, bytes32 inputHash, bytes memory proof) public onlyOwner(auctionId) {
        Auction storage auction = auctions[auctionId];
        require(auction.owner == msg.sender, "You need owner of auction");
        require(block.timestamp >= auctions[auctionId].endTime, "Auction has not ended yet");
        require(!auction.ended, "Auction has ended");
        require(_verifyProof(_winner, inputHash, proof), "User doesn't winner");
        require(_winner.price <= auction.depositPrice, "Winner has more bid price than deposit price");
        // Set winner and status auction
        auction.winner = _winner;
        auction.ended = true;
        // Send nft
        IERC721 nftContract = IERC721(auction.asset.nftContract);
        nftContract.safeTransferFrom(address(this), auction.winner.winner, auction.asset.tokenId);
        // Refund token
        if (auction.depositPrice > auction.winner.price) {
            lockToken.safeTransfer(auction.winner.winner, auction.depositPrice - auction.winner.price);
        }
        // Withdraw token
        lockToken.safeTransfer(msg.sender, auction.winner.price);

        emit AuctionEnded(auctionId, auction.winner.winner, auction.winner.price);
    }

    function withdraw(uint256 auctionId) public {
        Auction storage auction = auctions[auctionId];
        require(hasDeposited[auctionId][msg.sender], "No tokens to withdraw");
        require(auction.ended, "Tokens are still locked");
        // Transfer tokens from this contract to the user
        lockToken.safeTransfer(msg.sender, auction.depositPrice);
    }

    /**
     * @notice Verifies a zero-knowledge proof.
     */
    function _verifyProof(Winner memory winner, bytes32 inputHash, bytes memory proof) internal returns (bool) {
        // ZK-proof verification logic
        return true;
    }

    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes calldata data
    ) external pure override returns (bytes4) {
        return IERC721Receiver.onERC721Received.selector;
    }
}
