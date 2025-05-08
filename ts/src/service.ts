import { Service } from "zkwasm-ts-server";
import {TxWitness} from "zkwasm-ts-server/src/prover";
import {Event, EventModel} from "zkwasm-ts-server";
import { Position, IndexedObjectModel, IndexedObject, PositionModel, parseMemeInfo} from "./info.js";
import { Player} from "./api.js";
import { get_server_admin_key } from "zkwasm-ts-server/src/config.js";
import { Express } from "express";
//import {clearTxFromCommit, CommitModel, getTxFromCommit, insertTxIntoCommit} from "./commits.js";
import {merkleRootToBeHexString} from "zkwasm-ts-server/src/lib.js";
import { SanityService } from "./sanity_service.js";

const uncommittedTxs: TxWitness[] = [];


const uploadDir = "./uploads";
const sanityService = new SanityService(uploadDir);
const service = new Service(eventCallback, batchedCallback, extra);
await service.initialize();


let currentUncommitMerkleRoot: string = merkleRootToBeHexString(service.merkleRoot);

function extra (app: Express) {
	app.get('/data/position/:pid1/:pid2', async(req:any, res) => {
		let pid1:bigint = BigInt(req.params.pid1);
		let pid2:bigint = BigInt(req.params.pid2);
		console.log("query position:", pid1, pid2);
		let doc = await PositionModel.find(
				{pid_1: pid1, pid_2: pid2},
		);
		let data = doc.map((d) => {return Position.fromMongooseDoc(d).toJSON()})
		console.log("query position:", doc);
		
		

		res.status(200).send({
			success: true,
			data: data,
		});
	});
	app.get('/data/memes', async(req:any, res) => {
		const doc = await IndexedObjectModel.find();
		const jdoc = doc.map((d) => {
			const jdoc = IndexedObject.fromMongooseDoc(d);
			return parseMemeInfo(jdoc);
		});
		res.status(200).send({
			success: true,
			data: jdoc,
		});
	});
	sanityService.registerAPICallback(app);
}


await service.serve();
await sanityService.init();
await sanityService.setMemeList();

const EVENT_POSITION_UPDATE = 1;
const EVENT_MEME_UPDATE = 2;

let preemptcounter = 0;

async function bootstrap(merkleRoot: string): Promise<TxWitness[]> {
	/*
	const txs = await getTxFromCommit(merkleRoot);
	console.log("tsx in bootstrap:", txs);
	return txs;
	*/
	return [];
}

async function batchedCallback(arg: TxWitness[], preMerkle: string, postMerkle: string) {
	/*
	currentUncommitMerkleRoot = postMerkle;
	await clearTxFromCommit(currentUncommitMerkleRoot);
	preemptcounter = 0;
	*/
	return;
}

async function eventCallback(arg: TxWitness, data: BigUint64Array) {
	//insertTxIntoCommit(currentUncommitMerkleRoot, arg, preemptcounter);
	//preemptcounter ++;
	if(data.length == 0) {
		return;
	}

	//console.log("eventCallback", arg, data);
	if(data[0] != 0n) {
		console.log("non-zero return, tx failed", data[0]);
		return;
	}
	if(data.length <= 2) {
		console.log("no event data");
		return;
	}

	let event = new Event(data[1], data);
	let doc = new EventModel({
		id: event.id.toString(),
		data: Buffer.from(event.data.buffer)
	});

	try {
		let result = await doc.save();
		if (!result) {
			console.log("failed to save event");
			throw new Error("save event to db failed");
		}
	} catch(e) {
		console.log(e);
		console.log("event ignored");
	}
	let i = 2; // start pos
	while(i < data.length) {
		let eventType = Number(data[i]>>32n);
		let eventLength = data[i]&((1n<<32n)-1n);
		let eventData = data.slice(i+1, i+1+Number(eventLength));
		console.log("event", eventType, eventLength, eventData);
		switch(eventType) {
			case EVENT_POSITION_UPDATE:
				{
					console.log("position event");
					let position = Position.fromEvent(eventData);
					let doc = await PositionModel.findOneAndUpdate(
							{pid_1: position.pid_1, pid_2: position.pid_2, object_index: position.object_index},
							position.toObject(),
							{upsert: true}
					);
					console.log("save position", position.pid_1, position.pid_2, position.object_index);
				}
				break;
			case EVENT_MEME_UPDATE:
				{
					console.log("token event");
					let obj = IndexedObject.fromEvent(eventData);
					let doc = await IndexedObjectModel.findOneAndUpdate({index: obj.index}, obj.toObject(), {upsert: true});
					console.log("save token", doc);
				}
				break;
			default:
				console.log("unknown event");
				break;
		}
		i += 1 + Number(eventLength);
	}
}



