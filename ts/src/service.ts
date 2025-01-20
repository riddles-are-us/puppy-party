import { Service } from "zkwasm-ts-server";
import { SanityUploadService } from "./sanity_upload_service.js";

const uploadDir = "./uploads";
const sanityUploadService = new SanityUploadService(uploadDir);

const service = new Service(()=>{return;}, sanityUploadService.registerAPICallback);
service.initialize();
service.serve();


