// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ZkAuction} from "../src/ZkAuction.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run(address _lockToken) external returns (address) {
        vm.startBroadcast();

        ZkAuction new_zk_auction = new ZkAuction(_lockToken);

        vm.stopBroadcast();

        return address(new_zk_auction);
    }
}
