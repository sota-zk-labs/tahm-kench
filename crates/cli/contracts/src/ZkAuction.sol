// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";
import {ISP1Verifier} from "lib/sp1-contracts/contracts/src/ISP1Verifier.sol";

contract ZkAuction is IERC721Receiver {
    using SafeERC20 for IERC20;

    // Data for verifying proof
    bytes32 public constant VERIFICATION_KEY = 0x0016a6bd3516991b4d8e2c3e5b3022f2de3579c5a0ffa1b35ca53f9ed0bb4df1;
    ISP1Verifier public constant SP1_VERIFIER = ISP1Verifier(0x3B6041173B80E77f038f3F2C0f9744f04837185e);

    struct Auction {
        address owner; // Owner of the auction
        bytes encryptionKey; // Owner's public key
        IERC20 token; // Token used in the auction
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
        uint128 price; // Price submitted of winner
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
    event NewBid(
        uint256 indexed auctionId,
        address indexed bidder,
        bytes encryptedPrice
    );
    event AuctionEnded(
        uint256 indexed auctionId,
        address indexed winner,
        uint128 price
    );

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
        bytes memory _encryptionKey,
        IERC20 _token,
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
        newAuction.encryptionKey = _encryptionKey;
        newAuction.asset = Asset(
            _assetName,
            _assetDescription,
            _nftContract,
            _tokenId
        );
        newAuction.depositPrice = _depositPrice;
        newAuction.endTime = block.timestamp + _duration; // Set auction end time
        newAuction.ended = false;
        newAuction.token = _token;

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
        require(!hasDeposited[auctionId][msg.sender], "Already deposited");
        // Update the state to indicate that the user has deposited
        hasDeposited[auctionId][msg.sender] = true;
        // Bid
        auction.bids.push(
            Bid({bidder: msg.sender, encryptedPrice: _encryptedPrice})
        );

        auction.token.safeTransferFrom(
            msg.sender,
            address(this),
            auction.depositPrice
        );
        emit NewBid(auctionId, msg.sender, _encryptedPrice);
    }

    /**
     * @notice Gets list bidders after the bid phase end
     * @dev Uses auctionId to get list bidders.
     */
    function getBids(uint256 auctionId) public view returns (Bid[] memory) {
        require(
            block.timestamp >= auctions[auctionId].endTime,
            "Auction has not ended yet"
        );
        require(!auctions[auctionId].ended, "Auction has ended");
        return auctions[auctionId].bids;
    }

    /**
     * @notice Reveals the winner after the auction ends.
     * @dev Uses a ZK-proof to reveal the highest valid bid.
     */
    function finalizeAuction(
        uint256 auctionId,
        Winner memory _winner,
        bytes calldata publicInput,
        bytes memory proof
    ) public onlyOwner(auctionId) {
        Auction storage auction = auctions[auctionId];
        require(auction.owner == msg.sender, "You need owner of auction");
        require(
            block.timestamp >= auctions[auctionId].endTime,
            "Auction has not ended yet"
        );
        require(!auction.ended, "Auction has ended");
        _verifyProof(_winner, auctionId, publicInput, proof);
        require(
            _winner.price <= auction.depositPrice,
            "Winner has more bid price than deposit price"
        );
        // Set winner and status auction
        auction.winner = _winner;
        auction.ended = true;
        // Send nft
        IERC721 nftContract = IERC721(auction.asset.nftContract);
        nftContract.safeTransferFrom(
            address(this),
            auction.winner.winner,
            auction.asset.tokenId
        );
        // Refund token
        if (auction.depositPrice > auction.winner.price) {
            auction.token.safeTransfer(
                auction.winner.winner,
                auction.depositPrice - auction.winner.price
            );
        }
        // Withdraw token
        auction.token.safeTransfer(msg.sender, auction.winner.price);

        emit AuctionEnded(
            auctionId,
            auction.winner.winner,
            auction.winner.price
        );
    }

    function withdraw(uint256 auctionId) public {
        Auction storage auction = auctions[auctionId];
        require(hasDeposited[auctionId][msg.sender], "No tokens to withdraw");
        require(auction.ended, "Tokens are still locked");
        // Transfer tokens from this contract to the user
        auction.token.safeTransfer(msg.sender, auction.depositPrice);
    }

    function _verifyProof(
        Winner memory winner,
        uint256 auctionId,
        bytes calldata publicInput,
        bytes memory proof
    ) internal view {
        (
            bytes32 auctionHash,
            address winnerAddr,
            uint128 winnerPrice
        ) = decodePublicInput(publicInput);

        require(winnerAddr == winner.winner, "Winner address in proof does not match");
        require(winnerPrice == winner.price, "Winner price in proof does not match");
        require(
            calculateAuctionHash(auctionId) == auctionHash,
            "Auction hash does not match"
        );
        SP1_VERIFIER.verifyProof(VERIFICATION_KEY, publicInput, proof);
    }

    function decodePublicInput(bytes memory data) internal pure returns (
        bytes32 auctionHash,
        address winnerAddr,
        uint128 winnerPrice
    ) {
        auctionHash = bytes32(slice(data, 0, 32));
        winnerAddr = address(bytes20(slice(data, 32 + 8, 20)));
        winnerPrice = uint128(bytes16(reverse(slice(data, 32 + 8 + 20, 16))));
    }

    function slice(
        bytes memory data,
        uint256 start,
        uint256 length
    ) internal pure returns (bytes memory) {
        require(start + length <= data.length, "Slice out of bounds");
        bytes memory result = new bytes(length);
        for (uint256 i = 0; i < length; i++) {
            result[i] = data[start + i];
        }
        return result;
    }

    function reverse(bytes memory data) internal pure returns (bytes memory) {
        bytes memory result = new bytes(data.length);
        for (uint256 i = 0; i < data.length; i++) {
            result[i] = data[data.length - 1 - i];
        }
        return result;
    }

    function calculateAuctionHash(uint256 auctionId) internal view returns (bytes32) {
        Bid[] memory bids = auctions[auctionId].bids;
        bytes memory hashInput = abi.encodePacked(auctionId);
        for (uint256 i = 0; i < bids.length; ++i) {
            hashInput = abi.encodePacked(
                hashInput,
                bids[i].bidder,
                bids[i].encryptedPrice
            );
        }
        return keccak256(hashInput);
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
