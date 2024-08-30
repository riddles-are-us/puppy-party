import { query, ZKWasmAppRpc, LeHexBN } from "zkwasm-ts-server";

const CREATE_PLAYER = 1n;
const SHAKE_FEET = 2n;
const JUMP = 3n;
const SHAKE_HEADS = 4n;
const POST_COMMENTS = 5n;
const LOTTERY = 6n;

function createCommand(nonce: bigint, command: bigint) {
    return (nonce << 16n) + command;
}

let account = "1234";

const rpc = new ZKWasmAppRpc("http://localhost:3000");

async function testCreatePlayer() {
    let finished = 0;
    let accountInfo = new LeHexBN(query(account).pkx).toU64Array();
    console.log("account info:", accountInfo);
    try {
        let processStamp = await rpc.sendTransaction([createCommand(0n, CREATE_PLAYER), 0n, 0n, 0n], account);
        console.log("create player processed at:", processStamp);
    } catch(e) {
        let reason = "";
        if (e instanceof Error) {
          reason = e.message;
        }
        console.log("create player error:", reason);
    }

    try {
        let state = await rpc.queryState(account);
        console.log("query state:", state); 
    } catch(e) {
        console.log("query state error:", e);
    }
};

async function main() {
    testCreatePlayer();
}
main();
/*
test('create player already exists', () => {
    const cmd = createCommand(0, 1);
    const transaction = wasm.Transaction.decode([cmd, 0, 0, 0]);

    let res = transaction.create_player(PKEY);
    expect(res).toBe(0);
    res = transaction.create_player(PKEY);
    expect(res).toBe(1);
});

test('action works', () => {
    const cmd = createCommand(0, 1);
    const transaction = wasm.Transaction.decode([cmd, 0, 0, 0]);

    transaction.create_player(PKEY);

    let res = transaction.action(PKEY, 2);
    let player = wasm.PuppyPlayer.get(PKEY);
    expect(player.nonce).toBe(1);
    expect(player.data.current_action).toBe(2);
    expect(res).toBe(0);

    res = transaction.action(PKEY, 3);
    player = wasm.PuppyPlayer.get(PKEY);
    expect(player.nonce).toBe(2);
    expect(player.data.current_action).toBe(3);
    expect(res).toBe(0);

    res = transaction.action(PKEY, 4);
    player = wasm.PuppyPlayer.get(PKEY);
    expect(player.nonce).toBe(3);
    expect(player.data.current_action).toBe(4);
    expect(res).toBe(0);

    res = transaction.action(PKEY, 5);
    player = wasm.PuppyPlayer.get(PKEY);
    expect(player.nonce).toBe(4);
    expect(player.data.current_action).toBe(5);
    expect(res).toBe(0);

    res = transaction.action(PKEY, 6);
    player = wasm.PuppyPlayer.get(PKEY);
    expect(player.nonce).toBe(5);
    expect(player.data.current_action).toBe(6);
    expect(res).toBe(0);
});

test('action player not exist', () => {
    const cmd = createCommand(0, 1);
    const transaction = wasm.Transaction.decode([cmd, 0, 0, 0]);

    const res = transaction.action(PKEY, 2);
    expect(res).toBe(2);
});*/