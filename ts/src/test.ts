import { query, LeHexBN } from "zkwasm-ts-server";
import { Player } from "./api.js";

const CREATE_PLAYER = 1n;
const SHAKE_FEET = 2n;
const JUMP = 3n;
const SHAKE_HEADS = 4n;
const POST_COMMENTS = 5n;
const LOTTERY = 6n;

let account = "1234";
let player = new Player(account);


// Function to pause execution for a given duration
function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
    let accountInfo = new LeHexBN(query(account).pkx).toU64Array();
    console.log("account info:", accountInfo);

    console.log("Start run CREATE_PLAYER...");
    await player.runCommandAndCheckState(CREATE_PLAYER);

    console.log("Start run SHAKE_FEET...");
    let progress = await player.runCommandAndCheckState(SHAKE_FEET);
    await delay(15000); // Wait for 15 seconds/3 ticks

    console.log("Start run JUMP...");
    progress = await player.runCommandAndCheckState(JUMP);
    await delay(15000); // Wait for 15 seconds/3 ticks

    console.log("Start run SHAKE_HEADS...");
    progress = await player.runCommandAndCheckState(SHAKE_HEADS);
    await delay(15000); // Wait for 15 seconds/3 ticks

    console.log("Start run POST_COMMENTS...");
    progress = await player.runCommandAndCheckState(POST_COMMENTS);
    await delay(15000); // Wait for 15 seconds/3 ticks

    // Run extra 16 actions to test lottery
    for(let i = 0; i < 16; i++) {
      progress = await player.runCommandAndCheckState(JUMP);
      await delay(15000); // Wait for 15 seconds/3 ticks
    }
    console.log("Run extra actions done!");
    console.log("Start run LOTTERY...");
    await player.runCommandAndCheckState(LOTTERY);
}

main();