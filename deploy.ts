import spawn from 'cross-spawn';
import fs from 'fs';
import path from 'path';
import { Connection, LAMPORTS_PER_SOL, Keypair } from "@solana/web3.js"

function readKeyfile(keypairfile: any): Keypair {
  let kf = fs.readFileSync(keypairfile) as any;
  let parsed = JSON.parse(kf.toString()) // [1,1,2,2,3,4]
  kf = new Uint8Array(parsed)
  const keypair = Keypair.fromSecretKey(kf)
  return keypair
}

const main = async () => {
  const SLASH = path.sep;

  const authorityKeyfileName = `/Users/sotatek/my-solana-wallet/my-keypair.json`;
  const authorityKeypairFile = path.resolve(
    `${authorityKeyfileName}`
  )

  let authorityKeypair = readKeyfile(authorityKeypairFile);

  console.log(`Authority address: ${authorityKeypair.publicKey.toBase58()}`)

  const connection = new Connection("https://api.devnet.solana.com", "confirmed")

  const signature = await connection.requestAirdrop(
    authorityKeypair.publicKey,
    LAMPORTS_PER_SOL * 2
  )
  await connection.confirmTransaction(signature)

  console.log("Finish Airdrop");

  const accountBalance = await connection.getBalance(authorityKeypair.publicKey);
  console.log("Acount balance: " + accountBalance);

  const programKeyfileName = `target/deploy/solana_vesting-keypair.json`;
  const programKeypairFile = path.resolve(
    `${__dirname}${SLASH}${programKeyfileName}`
  )

  let programKeypair = readKeyfile(programKeypairFile)
  let programId = programKeypair.publicKey.toString()

  console.log(`programId: ${programId}`);

  let method = ["deploy"] // we are deploying for the first time, using 'deploy'

  try {
    await spawn.sync(
      "anchor",
      [
        ...method, // we use a variable so we when we want to upgrade, we can use 'upgrade' instead
        "--provider.cluster", // we want to specify the node cluster
        "Devnet", // the node cluster as the Devnet
        // "--provider.wallet", // we need to pass in a keyfile to pay for the deployment
        // `${authorityKeypair}`, // this is the keypair file we created just a moment ago
      ],
      { stdio: "inherit" }
    )
  } catch (err) {
    console.log(err);
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
