import { query, LeHexBN } from "zkwasm-ts-server";
import { Player } from "./api.js";

const CREATE_PLAYER = 1n;
const SHAKE_FEET = 2n;

let account = "1234";
let player = new Player(account);

async function testCreatePlayer() {
    console.log("Start run CREATE_PLAYER...");
    await player.runCommandAndCheckState(CREATE_PLAYER);
};

async function shakeFeet() {
    console.log("Start run SHAKE_FEET...");
    await player.runCommandAndCheckState(SHAKE_FEET);
};

async function main() {
    let accountInfo = new LeHexBN(query(account).pkx).toU64Array();
    console.log("account info:", accountInfo);

    await testCreatePlayer();
    await shakeFeet();
}

main();