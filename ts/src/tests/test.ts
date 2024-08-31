import { query, ZKWasmAppRpc, LeHexBN } from "zkwasm-ts-server";
import { Player } from "./api.js";

const CREATE_PLAYER = 1n;
const SHAKE_FEET = 2n;
const JUMP = 3n;
const SHAKE_HEADS = 4n;
const POST_COMMENTS = 5n;
const LOTTERY = 6n;

let account = "1234";
let player = new Player(account);

async function testCreatePlayer() {
    console.log("Start run CREATE_PLAYER...");
    await player.runCommand(CREATE_PLAYER, 0n);
    await player.checkState(0n, 0n, 0n);
};

async function testAction() {
    console.log("Start run SHAKE_FEET...");
    let nonce = await player.getNonce();
    await player.runCommand(SHAKE_FEET, nonce);
    await player.checkState(nonce + 1n, 0n, SHAKE_FEET);

    console.log("Start run JUMP...");
    nonce = await player.getNonce();
    await player.runCommand(JUMP, nonce);
    await player.checkState(nonce + 1n, 0n, JUMP);

    console.log("Start run SHAKE_HEADS...");
    nonce = await player.getNonce();
    await player.runCommand(SHAKE_HEADS, nonce);
    await player.checkState(nonce + 1n, 0n, SHAKE_HEADS);

    console.log("Start run POST_COMMENTS...");
    nonce = await player.getNonce();
    await player.runCommand(POST_COMMENTS, nonce);
    await player.checkState(nonce + 1n, 0n, POST_COMMENTS);

    console.log("Start run LOTTERY...");
    nonce = await player.getNonce();
    await player.runCommand(LOTTERY, nonce);
    await player.checkState(nonce + 1n, 0n, LOTTERY);
};

async function main() {
    let accountInfo = new LeHexBN(query(account).pkx).toU64Array();
    console.log("account info:", accountInfo);

    await testCreatePlayer();
    await testAction();
}
main();