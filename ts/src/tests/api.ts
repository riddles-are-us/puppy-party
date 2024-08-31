import { ZKWasmAppRpc } from "zkwasm-ts-server";

const CREATE_PLAYER = 1n;

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
  async checkState(nonce: bigint, name: bigint, current_action: bigint) {
    try {
      let data = await this.getState();
      let nonce_after_command = data[0][0].nonce;
      let state_name_after_command = data[0][0].data.name;
      let current_action_after_command = data[0][0].data.current_action;
      if(nonce == BigInt(nonce_after_command) && name == BigInt(state_name_after_command) && current_action == BigInt(current_action_after_command)) {
          console.log("command works");
      } else {
          console.log("command failed. current state's nonce:", current_action_after_command, ", name:", state_name_after_command, ", current_action:", current_action_after_command);
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
      let nonce_before_command = BigInt(data[0][0].nonce);
      await this.runCommand(command, BigInt(nonce_before_command));
      await this.checkState(nonce_before_command + 1n, 0n, command);
    }
  }
}