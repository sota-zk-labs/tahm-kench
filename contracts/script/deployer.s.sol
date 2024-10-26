// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";
import {ZkAuction} from "../src/ZkAuction.sol";
import {MockToken} from "../src/mocks/MockToken.sol";
import {MockNFT} from "../src/mocks/MockNFT.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run(address _lockToken) external returns (address) {
        vm.startBroadcast();

        ZkAuction new_zk_auction = new ZkAuction(_lockToken);
        MockToken new_mock_erc20 = new MockToken();
        MockNFT new_mock_nft = new MockNFT();

        vm.stopBroadcast();

        return address(new_zk_auction);
    }
}