// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";
import {ZkAuction} from "../src/ZkAuction.sol";
import {MockToken} from "../src/mocks/MockToken.sol";
import {MockNFT} from "../src/mocks/MockNFT.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run() external returns (address) {
        vm.startBroadcast();

        ZkAuction new_zk_auction = new ZkAuction();

        vm.stopBroadcast();

        return address(new_zk_auction);
    }
}