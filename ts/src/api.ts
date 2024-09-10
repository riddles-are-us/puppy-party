import { ZKWasmAppRpc } from "zkwasm-ts-server";

const SWAY = 0n;
const CREATE_PLAYER = 1n;
const LOTTERY = 6n;

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

  async runCommand(command: bigint, nonce: bigint) {
    try {
      console.log("command", command, "nonce", nonce);
      let processStamp = await rpc.sendTransaction([createCommand(command, nonce), 0n, 0n, 0n], this.processingKey);
      console.log("command processed at:", processStamp);
    } catch(e) {
      console.log(e)
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
      if(action == LOTTERY) {
        console.log("balance_after_command", balance_after_command, data);
      }
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