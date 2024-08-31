import { query, LeHexBN } from "zkwasm-ts-server";
import { Player } from "./api.js";

const CREATE_PLAYER = 1n;
const POST_COMMENTS = 5n;

let account = "1234";
let player = new Player(account);

async function testCreatePlayer() {
    console.log("Start run CREATE_PLAYER...");
    await player.runCommandAndCheckState(CREATE_PLAYER);
};

async function postComments() {
    console.log("Start run POST_COMMENTS...");
    await player.runCommandAndCheckState(POST_COMMENTS);
};

async function main() {
    let accountInfo = new LeHexBN(query(account).pkx).toU64Array();
    console.log("account info:", accountInfo);

    await testCreatePlayer();
    await postComments();
}

main();