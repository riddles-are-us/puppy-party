import express, { Express } from "express";
import { LeHexBN, query, ZKWasmAppRpc } from "zkwasm-ts-server";
import cors from "cors";
import multer, { Multer } from "multer";
import path from "path";
import fs from "fs";
import dotenv from "dotenv";
import { createClient, SanityClient } from "@sanity/client";
import { Player } from "./api.js";

const INSTALL_PLAYER = 1n;
const INSTALL_MEME = 7n;
const WITHDRAW = 8n;
const DEPOSIT = 9n;
let account = "1234";
const rpc: any = new ZKWasmAppRpc("http://127.0.0.1:3000");
let player = new Player(account, rpc, DEPOSIT, WITHDRAW);

export class SanityUploadService {
  sanityClient: SanityClient;
  multer: Multer;
  registerAPICallback: (app: Express) => void;
  nonce: bigint;

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
  }

  async init() {
    await player.runCommand(INSTALL_PLAYER, 0n, []);
    this.nonce = await player.getNonce();
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

          const newDocument = {
            _type: "meme",
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
          player.runCommand(INSTALL_MEME, this.nonce, []);

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
