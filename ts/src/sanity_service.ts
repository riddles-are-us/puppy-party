import express, { Express } from "express";
import { LeHexBN, query, ZKWasmAppRpc } from "zkwasm-ts-server";
import cors from "cors";
import multer, { Multer } from "multer";
import path from "path";
import fs from "fs";
import dotenv from "dotenv";
import { createClient, SanityClient } from "@sanity/client";
import { Player } from "./api.js";
import sanityClient from "./sanityClient.js";
import { IndexedObject, IndexedObjectModel, parseMemeInfo } from "./info.js";

const INSTALL_PLAYER = 1n;
const INSTALL_MEME = 7n;
const WITHDRAW = 8n;
const DEPOSIT = 9n;
let account = "1234";
const rpc: any = new ZKWasmAppRpc("http://127.0.0.1:3000");
let player = new Player(account, rpc, DEPOSIT, WITHDRAW);

export class SanityService {
  sanityClient: SanityClient;
  multer: Multer;
  registerAPICallback: (app: Express) => void;
  nonce: bigint;
  latestId: number;

  constructor(uploadDir: string) {
    if (!fs.existsSync(uploadDir)) {
      fs.mkdirSync(uploadDir, { recursive: true });
    }
    const storage = multer.diskStorage({
      destination: (req, file, cb) => {
        cb(null, uploadDir);
      },
      filename: (req, file, cb) => {
        cb(null, Date.now() + path.extname(file.originalname));
      },
    });

    this.multer = multer({ storage: storage });
    dotenv.config();
    this.sanityClient = createClient({
      projectId: "wl3vyz0o",
      dataset: "production",
      apiVersion: "2023-01-01",
      useCdn: true,
      token: process.env.SANITY_TOKEN,
    });

    this.registerAPICallback = (app: Express) => {
      this.registerAPI(app);
    };

    this.nonce = 0n;
    this.latestId = 0;
  }

  async init() {
    await player.runCommand(INSTALL_PLAYER, 0n, []);
    this.nonce = await player.getNonce();
    const query = `
		*[_type == "meme"] | order(id desc)[0]{
	id
}`;

    const latestMeme: any = await sanityClient
      .fetch(query)
      .catch((error: any) => {
        console.error("Error fetching data:", error);
      });
    this.latestId = latestMeme.id;
  }

  async setMemeList() {
    const query = `
		*[_type == "season" && (isCurrentSeason == true || isPreviousSeason == true)] {
			name,
			seasonEndDate,
			"isCurrentSeason": coalesce(isCurrentSeason, false),
			"isPreviousSeason": coalesce(isPreviousSeason, false),
			"memes": coalesce(memes[]->{
				id,
				name,
				"avatar": avatar.asset->url,
				"spriteSheet": spriteSheet.asset->url
			}, [])
		}`;

    const seasonDatas: SeasonData[] = await sanityClient
      .fetch(query)
      .catch((error: any) => {
        console.error("Error fetching data:", error);
      });

    const currentSeason = seasonDatas.find((season) => season.isCurrentSeason);
    console.log("currentSeason", currentSeason);

    const doc = await IndexedObjectModel.find();
    const idSet = new Set(
      doc.map((d) => {
        const jdoc = IndexedObject.fromMongooseDoc(d);
        return parseMemeInfo(jdoc).id;
      })
    );
    console.log("idSet", idSet);

    if (currentSeason) {
      currentSeason.memes.forEach((meme) => {
        if (!idSet.has(meme.id)) {
          player.runCommand(INSTALL_MEME, this.nonce, [BigInt(meme.id)]);
        }
      });
    }
  }

  private registerAPI(app: Express) {
    app.post(
      "/upload",
      this.multer.fields([
        { name: "avatar", maxCount: 1 },
        { name: "spriteSheet", maxCount: 1 },
      ]),
      async (req: any, res) => {
        try {
          const { name } = req.body;
          const files = req.files as {
            [fieldname: string]: Express.Multer.File[];
          };

          if (!name || !files.avatar || !files.spriteSheet) {
            return res.status(400).json({ error: "Missing required fields" });
          }

          // Upload avatar to Sanity
          const avatarFile = files.avatar[0];
          const avatarAsset = await this.sanityClient.assets.upload(
            "image",
            fs.createReadStream(avatarFile.path),
            {
              filename: avatarFile.originalname,
            }
          );

          // Upload spriteSheet to Sanity
          const spriteSheetFile = files.spriteSheet[0];
          const spriteSheetAsset = await this.sanityClient.assets.upload(
            "image",
            fs.createReadStream(spriteSheetFile.path),
            {
              filename: spriteSheetFile.originalname,
            }
          );

          const id = ++this.latestId;
          const newDocument = {
            _type: "meme",
            id,
            name: name,
            avatar: {
              _type: "image",
              asset: {
                _ref: avatarAsset._id,
              },
            },
            spriteSheet: {
              _type: "image",
              asset: {
                _ref: spriteSheetAsset._id,
              },
            },
          };

          const createdDocument = await this.sanityClient.create(newDocument);
          await player.runCommand(INSTALL_MEME, this.nonce, [BigInt(id)]);

          fs.unlinkSync(avatarFile.path);
          fs.unlinkSync(spriteSheetFile.path);

          res
            .status(200)
            .json({ message: "Upload successful!", document: createdDocument });
        } catch (error) {
          console.error("Error uploading to Sanity:", error);
          res.status(500).json({ error: "Internal server error" });
        }
      }
    );
  }
}
export interface SeasonData {
  name: string;
  seasonEndDate: string;
  isCurrentSeason: boolean;
  isPreviousSeason: boolean;
  memes: MemeData[];
}

export interface MemeData {
  id: number;
  name: string;
  avatar: string;
  spriteSheet: string;
  rank: number;
}
