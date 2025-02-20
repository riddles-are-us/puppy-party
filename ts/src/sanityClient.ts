import { createClient } from "@sanity/client";

const sanityClient: any = createClient({
  projectId: "wl3vyz0o",
  dataset: "production",
  apiVersion: "2023-01-01",
  useCdn: true,
});

export default sanityClient;
