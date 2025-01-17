import express from "express";
import cors from "cors";
import multer from "multer";
import path from "path";

const app = express();

// Configure multer for file uploads
const storage = multer.diskStorage({
  destination: (req, file, cb) => {
    cb(null, "uploads/"); // Save files to the 'uploads' directory
  },
  filename: (req, file, cb) => {
    cb(null, Date.now() + path.extname(file.originalname)); // Use current timestamp as filename
  },
});

const upload = multer({ storage: storage });

// Enable CORS
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

// POST route to handle image uploads
app.post("/upload", upload.fields([{ name: "avatar", maxCount: 1 }, { name: "spriteSheet", maxCount: 1 }]), async (req: any, res) => {
    try {
        // Access the text fields from the request body
        const { name } = req.body;

        // Access the uploaded files from the request
        const avatar = req.files?.avatar ? (req.files.avatar as Express.Multer.File[])[0].path : null;
        const spriteSheet = req.files?.spriteSheet ? (req.files.spriteSheet as Express.Multer.File[])[0].path : null;

        // Log the received data
        console.log("Received data:", { name, avatar, spriteSheet });

        // Respond with the success message and the file paths
        res.status(200).json({
            message: "Upload successful!",
            receivedData: { name, avatar, spriteSheet },
        });
    } catch (error: any) {
        console.error("Error processing request:", error);
        res.status(500).json({ error: error.message || "Internal server error" });
    }
});

// Set up the server to listen on port 3030
const PORT = 3030;
app.listen(PORT, () => {
    console.log(`Server is running on http://localhost:${PORT}`);
});
