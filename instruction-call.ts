import * as spl from '@solana/spl-token';
import * as anchor from "@project-serum/anchor";

type Keypair = anchor.web3.Keypair;
type PublicKey = anchor.web3.PublicKey;

const main = async () => {
  let ruinStakingTreasury: Keypair;
  let staker: Keypair;
  let deployerKeypair: Keypair;
  let deployer: PublicKey;

  deployerKeypair = anchor.web3.Keypair.generate();
  deployer = deployerKeypair.publicKey;
  staker = anchor.web3.Keypair.generate();
  ruinStakingTreasury = anchor.web3.Keypair.generate();

  console.log(deployerKeypair.publicKey.toBase58());
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

