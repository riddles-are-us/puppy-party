import express from "express";
import cors from "cors";
import multer from "multer";
import path from "path";
import fs from "fs";
import dotenv from "dotenv";
import { createClient } from "@sanity/client";

// Multer storage configuration
const uploadDir = "./uploads";
console.log("upload dir", uploadDir)
if (!fs.existsSync(uploadDir)) {
  fs.mkdirSync(uploadDir, { recursive: true });
}

dotenv.config();
const sanityClient = createClient({
  projectId: "wl3vyz0o",
  dataset: "production",
  apiVersion: "2023-01-01",
  useCdn: true,
  token: process.env.SANITY_TOKEN
});

const PORT = 3030;
const storage = multer.diskStorage({
  destination: (req, file, cb) => {
    cb(null, "uploads/"); 
  },
  filename: (req, file, cb) => {
    cb(null, Date.now() + path.extname(file.originalname)); 
  },
});
const upload = multer({ storage: storage });

const app = express();
app.use(cors({
    origin: (origin, callback) => {
        const allowedOrigins = [/^http:\/\/localhost(:\d+)?$/, "https://disco.zkplay.app/"];
        
        if (!origin || allowedOrigins.some((allowed) => allowed instanceof RegExp ? allowed.test(origin) : allowed === origin)) {
            callback(null, true);
        } else {
            callback(new Error("Not allowed by CORS"));
        }
    },
    methods: ["GET", "POST"],
    allowedHeaders: ["Content-Type"],
}));
app.use(express.json());
app.use(express.urlencoded({ extended: true }));
app.post("/upload", upload.fields([{ name: "avatar", maxCount: 1 }, { name: "spriteSheet", maxCount: 1 }]), async (req: any, res) => {
    
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
        const avatarAsset = await sanityClient.assets.upload("image", fs.createReadStream(avatarFile.path), {
          filename: avatarFile.originalname,
        });
  
        // Upload spriteSheet to Sanity
        const spriteSheetFile = files.spriteSheet[0];
        const spriteSheetAsset = await sanityClient.assets.upload("image", fs.createReadStream(spriteSheetFile.path), {
          filename: spriteSheetFile.originalname,
        });
  
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
  
        const createdDocument = await sanityClient.create(newDocument);
  
        fs.unlinkSync(avatarFile.path);
        fs.unlinkSync(spriteSheetFile.path);
  
        res.status(200).json({ message: "Upload successful!", document: createdDocument });
      } catch (error) {
        console.error("Error uploading to Sanity:", error);
        res.status(500).json({ error: "Internal server error" });
      }
});
app.listen(PORT, () => {
    console.log(`Server is running on http://localhost:${PORT}`);
});
