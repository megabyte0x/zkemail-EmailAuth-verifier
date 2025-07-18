import zkeSDK from "@zk-email/sdk";
import fs from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";

// Copy slug from UI homepage
const blueprintSlug = "DimiDumo/SuccinctZKResidencyInvite@v3";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
  const sdk = zkeSDK();

  // Get an instance of Blueprint
  const blueprint = await sdk.getBlueprint(blueprintSlug);

  // Create a prover from the blueprint
  const prover = blueprint.createProver();

  // Get eml
  const eml = (await fs.readFile(path.join(__dirname, "../emls/residency.eml"))).toString();

  // Generate and wait until proof is generated, can take up to a few minutes
  const proof = await prover.generateProof(eml);
  const { proofData, publicData } = proof.getProofData();

  // Save proofData into ../../test/proofData.json as JSON
  await fs.writeFile(
    path.join(__dirname, "../../test/proofData.json"),
    JSON.stringify(proofData)
  );
  // Save publicData into ../../test/publicData.json as JSON
  await fs.writeFile(
    path.join(__dirname, "../../test/publicData.json"),
    JSON.stringify(publicData)
  );

  console.log("Completed")



  // const vk = await blueprint.getVkey();


  // // Save vk into ../../test/vkey.json
  // await fs.writeFile(path.join(__dirname, "../../test/vkey.json"), vk);

  return;
}

main();
