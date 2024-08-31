import { ZKWasmAppRpc } from "zkwasm-ts-server";

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

  async getNonce(): Promise<bigint> {
    const data = await this.getState();
    let nonce = BigInt(data[0][0].nonce);
    return nonce;
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
      let state_nonce = data[0][0].nonce;
      let state_name = data[0][0].data.name;
      let state_current_action = data[0][0].data.current_action;
      if(nonce == BigInt(state_nonce) && name == BigInt(state_name) && current_action == BigInt(state_current_action)) {
          console.log("command works");
      } else {
          console.log("command failed. current state's nonce:", state_nonce, ", name:", state_name, ", current_action:", state_current_action);
      }
    } catch(e) {
      console.log("query state error:", e);
    }
  }
}