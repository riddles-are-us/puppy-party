import { query, LeHexBN } from "zkwasm-ts-server";
import { Player } from "./api.js";

const CREATE_PLAYER = 1n;
const SHAKE_HEADS = 4n;

let account = "1234";
let player = new Player(account);

async function testCreatePlayer() {
    console.log("Start run CREATE_PLAYER...");
    await player.runCommandAndCheckState(CREATE_PLAYER);
};

async function shakeHeads() {
    console.log("Start run SHAKE_HEADS...");
    await player.runCommandAndCheckState(SHAKE_HEADS);
};

async function main() {
    let accountInfo = new LeHexBN(query(account).pkx).toU64Array();
    console.log("account info:", accountInfo);

    await testCreatePlayer();
    await shakeHeads();
}

main();