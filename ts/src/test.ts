import { Player } from "./api.js";
//import { LeHexBN, ZKWasmAppRpc} from "zkwasm-minirollup-rpc";
import { LeHexBN, query, ZKWasmAppRpc} from "zkwasm-ts-server";
import { createAsyncThunk } from '@reduxjs/toolkit';

const INSTALL_PLAYER = 1n;
const VOTE = 2n;
const STAKE = 3n;
const COLLECT = 4n;
const COMMENT = 5n;
const LOTTERY = 6n;
const INSTALL_MEME = 7n;
const WITHDRAW = 8n;
const DEPOSIT = 9n;
const WITHDRAW_LOTTERY = 10n;



let account = "1234";

const rpc:any = new ZKWasmAppRpc("http://127.0.0.1:3000");
let player = new Player(account, rpc, DEPOSIT, WITHDRAW);

// Function to pause execution for a given duration
function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {

    const pubkey = new LeHexBN(query(account).pkx).toU64Array();
    console.log(pubkey);

    let r = await player.rpc.queryConfig();
    console.log(r);
    console.log("Start run CREATE_PLAYER...");
    await player.runCommand(INSTALL_PLAYER, 0n, []);

    let nonce = await player.getNonce();
    await player.runCommand(VOTE, nonce, [0n]);

    nonce = await player.getNonce();
    await player.runCommand(VOTE, nonce, [1n]);


    nonce = await player.getNonce();
    await player.runCommand(INSTALL_MEME, nonce, [1n]);


    nonce = await player.getNonce();
    await player.runCommand(INSTALL_MEME, nonce, [2n]);


    nonce = await player.getNonce();
    await player.runCommand(INSTALL_MEME, nonce, [3n]);

    nonce = await player.getNonce();
    await player.runCommand(STAKE, nonce, [1n, 1n]);

    nonce = await player.getNonce();
    await player.runCommand(STAKE, nonce, [0n, 2n]);

    nonce = await player.getNonce();
    await player.runCommand(VOTE, nonce, [1n]);

    let data = await player.rpc.queryData(`position/${pubkey[1]}/${pubkey[2]}`)
    console.log(data);

    nonce = await player.getNonce();
    await player.runCommand(COLLECT, nonce, [1n]);


    data = await player.rpc.queryData(`position/${pubkey[1]}/${pubkey[2]}`)
    console.log(data);

    data = await player.rpc.queryData(`memes`);

    console.log(data);





    /*
    console.log("Start run SHAKE_FEET...");
    await player.runCommandAndCheckState(SHAKE_FEET);
    await delay(10000); // Wait for 10 seconds/2 ticks

    console.log("Start run JUMP...");
    await player.runCommandAndCheckState(JUMP);
    await delay(10000); // Wait for 10 seconds/2 ticks

    console.log("Start run SHAKE_HEADS...");
    await player.runCommandAndCheckState(SHAKE_HEADS);
    await delay(10000); // Wait for 10 seconds/2 ticks

    console.log("Start run POST_COMMENTS...");
    await player.runCommandAndCheckState(POST_COMMENTS);
    await delay(10000); // Wait for 10 seconds/2 ticks

    // Run extra 16 actions to test lottery
    for(let i = 0; i < 16; i++) {
      await player.runCommandAndCheckState(JUMP);
      await delay(10000); // Wait for 10 seconds/2 ticks
    }
    console.log("Run extra actions done!");
    console.log("Start run LOTTERY...");
    await player.runCommandAndCheckState(LOTTERY);

    console.log("Start run WITHDRAW...");
    await player.runCommandAndCheckState(WITHDRAW);
    */
}

main();
