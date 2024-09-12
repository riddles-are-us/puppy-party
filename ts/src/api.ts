import { ZKWasmAppRpc } from "zkwasm-ts-server";
import BN from "bn.js";

const SWAY = 0n;
const CREATE_PLAYER = 1n;
const LOTTERY = 6n;
const WITHDRAW = 8n;

// for withdraw
const address = "c177d1d314C8FFe1Ea93Ca1e147ea3BE0ee3E470";
const amount = 1n;

function bytesToHex(bytes: Array<number>): string  {
  return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
}

function createCommand(command: bigint, nonce: bigint) {
  return (nonce << 16n) + command;
}

const rpc = new ZKWasmAppRpc("http://localhost:3000");

export class Player {
  processingKey: string;
  constructor(key: string) {
    this.processingKey = key
  }

  async getState(): Promise<any> {
    // Get the state response
    let state = await rpc.queryState(this.processingKey);

    // Parse the response to ensure it is a plain JSON object
    const parsedState = JSON.parse(JSON.stringify(state));

    // Extract the data from the parsed response
    const data = JSON.parse(parsedState.data);

    return data;
  }

  async withdrawRewards(address: string, amount: bigint, nonce: bigint) {
    let addressBN = new BN(address, 16);
    let addressBE = addressBN.toArray("be", 20); // 20 bytes = 160 bits and split into 4, 8, 8
    console.log("address is", address);
    console.log("address big endian is", addressBE);
    let firstLimb = BigInt('0x' + bytesToHex(addressBE.slice(0,4).reverse()));
    let sndLimb = BigInt('0x' + bytesToHex(addressBE.slice(4,12).reverse()));
    let thirdLimb = BigInt('0x' + bytesToHex(addressBE.slice(12, 20).reverse()));

    /*
    (32 bit amount | 32 bit highbit of address)
    (64 bit mid bit of address (be))
    (64 bit tail bit of address (be))
    */

    console.log("first is", firstLimb);
    console.log("snd is", sndLimb);
    console.log("third is", thirdLimb);

    try {
      let processStamp = await rpc.sendTransaction([
        createCommand(WITHDRAW, nonce),
        (firstLimb << 32n) + amount,
        sndLimb,
        thirdLimb
      ], this.processingKey);
      console.log("withdraw rewards processed at:", processStamp);
    } catch(e) {
      if (e instanceof Error) {
        console.log(e.message);
      }
      console.log("collect reward error at address:", address);
    }
  }

  async runCommand(command: bigint, nonce: bigint) {
    try {
      let processStamp = await rpc.sendTransaction([createCommand(command, nonce), 0n, 0n, 0n], this.processingKey);
      console.log("command processed at:", processStamp);
    } catch(e) {
      let reason = "";
      if (e instanceof Error) {
        reason = e.message;
      }
      console.log("command error:", reason);
    }
  }

  // Check whether the current state is as expected
  async checkState(nonce: bigint, action: bigint, balance: bigint) {
    try {
      let data = await this.getState();
      let nonce_after_command = data.player.nonce;
      let balance_after_command = data.player.data.balance;
      let action_after_command = data.player.data.action;

      if(nonce == BigInt(nonce_after_command) && action == BigInt(action_after_command) && balance == BigInt(balance_after_command)) {
          console.log("command works");
      } else {
          console.log("command failed. current state's nonce:", nonce_after_command, ", balance:", balance_after_command, ", action:", action_after_command);
      }
    } catch(e) {
      console.log("query state error:", e);
    }
  }

  async runCommandAndCheckState(command: bigint) {
    if(command == CREATE_PLAYER) {
      await this.runCommand(command, 0n);
      await this.checkState(0n, 0n, 0n);
    } else if(command == WITHDRAW){
      let data = await this.getState();
      let nonce_before_command = BigInt(data.player.nonce);
      let balance = 9n // previous balance(10n) - amount(1n)
      await this.withdrawRewards(address, amount, nonce_before_command);
      await this.checkState(nonce_before_command + 1n, SWAY, balance);
    } else {
      let data = await this.getState();
      let nonce_before_command = BigInt(data.player.nonce);
      await this.runCommand(command, BigInt(nonce_before_command));
      let balance = 0n;

      // Run lottery once on test.ts, so balance is 10, action become SWAY
      if(command == LOTTERY) {
        balance = 10n;
        await this.checkState(nonce_before_command + 1n, SWAY, balance);
      } else {
        await this.checkState(nonce_before_command + 1n, command, balance);
      }
    }
  }
}