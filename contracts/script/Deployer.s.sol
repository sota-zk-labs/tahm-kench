// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ZkAuction} from "../src/ZkAuction.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run() external returns (address) {
        vm.startBroadcast();

        ZkAuction zk_auction = new ZkAuction();

        vm.stopBroadcast();

        return address(zk_auction);
    }
}
