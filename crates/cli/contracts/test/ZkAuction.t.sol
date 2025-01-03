// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {Test, console} from "forge-std/Test.sol";
import {ZkAuction} from "../src/ZkAuction.sol";
import {MockToken} from "../src/mocks/MockToken.sol";
import {MockNFT} from "../src/mocks/MockNFT.sol";

contract testZkAuction is Test {
    ZkAuction public zk_auction;
    MockNFT public mock_nft;
    MockToken public mock_token;

    // Parameters for auction
    address public owner = address(0xdc83161e1f2864E727f7aDBFaF80CF9137e4e78d);
    address public bidder = address(0xbB2D3aF614081D504f6d929ba7558AE7cc0d2899);
    bytes public ownerPublicKey = "0xddaAd340b0f1Ef65169Ae5E41A8b10776a75482d";
    bytes public encryptedPrice = "0xddaAd340b0f1Ef65169Ae5E41A8b10776a75482d";
    string public assetName = "test01";
    string public assetDescription = "Test Asset Description";
    uint256 public tokenId_1 = 1;
    uint256 public tokenId_2 = 2;
    uint128 public depositPrice = 1 * 1e18;
    uint256 public duration = 3600; // 1 hour

    function setUp() public {
        // Deploy new erc20_mock and erc721_mock contracts
        mock_token = new MockToken();
        mock_nft = new MockNFT();

        // Deploy a new ZkAuction Contract instance
        zk_auction = new ZkAuction();

        // Mint an NFT to the owner for testing
        vm.prank(owner);
        mock_nft.mint(owner, tokenId_1);
        mock_nft.mint(owner, tokenId_2);

        // Approve NFT to contract for testing
        vm.prank(owner);
        mock_nft.approve(address(zk_auction), tokenId_1);
    }

    function createAuction() public {
        vm.prank(owner);
        zk_auction.createAuction(
            ownerPublicKey, IERC20(address(mock_token)), address(mock_nft), tokenId_1, assetName, assetDescription, depositPrice, duration
        );
    }

    function create_new_bid() public {
        // Mint token to the bidder for testing
        vm.prank(bidder);
        mock_token.mint(bidder, depositPrice);

        // Approve token to contract for testing
        vm.prank(bidder);
        mock_token.approve(address(zk_auction), depositPrice);

        vm.prank(bidder);
        zk_auction.placeBid(0, encryptedPrice);
    }

    // Test initial set up
    function testCreateAuctionSuccess() public {
        createAuction();
        // Ensure NFT has been transferred to the auction contract
        assertEq(mock_nft.ownerOf(tokenId_1), address(zk_auction));
        assertEq(zk_auction.auctionCount(), 1);

        (address address_owner, bytes memory pk,,,, uint256 deposit_price, uint256 endTime, bool ended) =
                            zk_auction.auctions(0);
        assertEq(address_owner, owner);
        assertEq(pk, ownerPublicKey);
        assertEq(deposit_price, depositPrice);
        assertEq(endTime, block.timestamp + duration);
        assertEq(ended, false);
    }

    function testCreateAuctionDepositPriceZero() public {
        vm.prank(owner);
        vm.expectRevert(bytes("Deposit price must be greater than zero"));
        zk_auction.createAuction(ownerPublicKey, IERC20(address(mock_token)), address(mock_nft), tokenId_1, assetName, assetDescription, 0, duration);
    }

    function testCreateAuctionDurationZero() public {
        vm.prank(owner);
        vm.expectRevert(bytes("Duration must be greater than zero"));
        zk_auction.createAuction(
            ownerPublicKey, IERC20(address(mock_token)), address(mock_nft), tokenId_1, assetName, assetDescription, depositPrice, 0
        );
    }

    function testCreateAuctionCallerNotOwnNFT() public {
        vm.prank(bidder);
        vm.expectRevert(bytes("You must own the NFT to auction it"));
        zk_auction.createAuction(
            ownerPublicKey, IERC20(address(mock_token)), address(mock_nft), tokenId_1, assetName, assetDescription, depositPrice, duration
        );
    }

    function testCreateAuctionNotApproveNFT() public {
        vm.prank(owner);
        vm.expectRevert(bytes("You need approve the NFT to contract"));
        zk_auction.createAuction(
            ownerPublicKey, IERC20(address(mock_token)), address(mock_nft), tokenId_2, assetName, assetDescription, depositPrice, duration
        );
    }

    // Test bid phase
    function testCreateBidAuctionEnded() public {
        createAuction();

        (,,,,,, uint256 endTime, bool ended) = zk_auction.auctions(0);

        assertTrue(block.timestamp < endTime);
        assertFalse(ended);

        // Mint token to the bidder for testing
        vm.prank(bidder);
        mock_token.mint(bidder, depositPrice);

        // Approve token to contract for testing
        vm.prank(bidder);
        mock_token.approve(address(zk_auction), depositPrice);

        // Set up block.timestamp
        vm.warp(block.timestamp + duration + 1);
        vm.prank(bidder);
        vm.expectRevert(bytes("Auction has expired"));
        zk_auction.placeBid(1, encryptedPrice);
    }

    function testDoubleCreateBid() public {
        createAuction();
        create_new_bid();
        vm.prank(bidder);
        vm.expectRevert(bytes("Already deposited"));
        zk_auction.placeBid(0, encryptedPrice);
    }

    // Test verify phase
    // Test follow time logic
}
