import * as spl from '@solana/spl-token';
import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import { createMint, createTokenAccount, sleep } from "@project-serum/common";
import { SolanaVesting } from "../target/types/solana_vesting";
import { mintToAccount } from "./utilities";
import * as assert from "assert";

type Keypair = anchor.web3.Keypair;
type PublicKey = anchor.web3.PublicKey;

describe("solana-staking", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.SolanaVesting as Program<SolanaVesting>;

  const LOCK_DURATION = 5;

  let ruinStaking: Keypair;
  let ruinStakingTerm: Keypair;
  let ruinStakingTreasury: Keypair;
  let staker: Keypair;
  let deployerKeypair: Keypair;
  let vaultAuthority: PublicKey;
  let userPendingWithdrawl: PublicKey;
  let userStaked: PublicKey;
  let deployer: PublicKey;
  let stakingToken: PublicKey;
  let distributorTokenAccount: PublicKey;
  let stakerTokenAccount: PublicKey;

  beforeEach(async () => {
    deployerKeypair = anchor.web3.Keypair.generate();
    deployer = deployerKeypair.publicKey;
    staker = anchor.web3.Keypair.generate();
    ruinStakingTreasury = anchor.web3.Keypair.generate();
    ruinStaking = anchor.web3.Keypair.generate();
    ruinStakingTerm = anchor.web3.Keypair.generate();

    const signature = await program.provider.connection.requestAirdrop(deployer, 90000000000000);
    await program.provider.connection.confirmTransaction(signature, 'confirmed');

    const transferTransaction = new web3.Transaction()
      .add(anchor.web3.SystemProgram.transfer({
        fromPubkey: deployer,
        toPubkey: staker.publicKey,
        lamports: 40000000000000
      }))

    await program.provider.connection.sendTransaction(
      transferTransaction,
      [deployerKeypair]
    );


    stakingToken = await createMint(
      program.provider,
      deployer,
      6
    )

    stakerTokenAccount = await createTokenAccount(
      program.provider,
      stakingToken,
      staker.publicKey,
    )

    await mintToAccount(
      program.provider,
      stakingToken,
      stakerTokenAccount,
      "9000000000000",
      deployer,
      deployerKeypair
    );


    let [distributorPubkey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("distributor"),
        stakingToken.toBuffer(),
        deployer.toBuffer(),
        new anchor.BN(LOCK_DURATION).toArrayLike(Buffer),
      ],
      program.programId,
    );

    distributorTokenAccount = distributorPubkey;

    // ------------------         -------------------------
    let [userStakedPubkey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("stake"),
        ruinStaking.publicKey.toBuffer(),
        ruinStakingTerm.publicKey.toBuffer(),
        staker.publicKey.toBuffer()
      ],
      program.programId,
    );

    let [pendingWithdrawlPubkey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("withdraw_reward"),
        ruinStaking.publicKey.toBuffer(),
        ruinStakingTerm.publicKey.toBuffer(),
        staker.publicKey.toBuffer()
      ],
      program.programId,
    );

    let [vaultAuthorityPubkey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("vault-authority"),
      ],
      program.programId,
    );

    const startJoinTime = Math.floor(new Date().getTime() / 1000) - 3000;
    const endJointTime = startJoinTime + 50000;

    try {
      await program.rpc.initialize(
        new anchor.BN(10),
        new anchor.BN(200 * (10 ** 6)),
        new anchor.BN(LOCK_DURATION),
        new anchor.BN(startJoinTime),
        new anchor.BN(endJointTime),
        new anchor.BN(10),
        new anchor.BN(40000),
        new anchor.BN(5),
        {
          accounts: {
            ruinStakingTreasury: ruinStakingTreasury.publicKey,
            ruinStakingTerm: ruinStakingTerm.publicKey,
            ruinStaking: ruinStaking.publicKey,
            ruinStakingAdmin: deployer,
            ruinStakingToken: stakingToken,
            ruinStakingDistributor: distributorTokenAccount,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
          signers: [
            deployerKeypair,
            ruinStakingTreasury,
            ruinStaking,
            ruinStakingTerm,
          ]
        }
      )
    } catch (err) {
      console.log(err);
    }

    userStaked = userStakedPubkey;
    userPendingWithdrawl = pendingWithdrawlPubkey;
    vaultAuthority = vaultAuthorityPubkey;

    await mintToAccount(
      program.provider,
      stakingToken,
      distributorTokenAccount,
      "9000000000000",
      deployer,
      deployerKeypair
    );
  });

  it("Staking info system can be initialized", async () => {
    const term = await program.account.ruinStakingTerm.fetch(ruinStakingTerm.publicKey);

    assert.equal(ruinStaking.publicKey.toBase58(), term.ruinStaking.toBase58());
    assert.equal(term.apr, 40000);
  });

  it("Staking info system can't be initialized if lacks any signature", async () => {
    try {
      await program.rpc.initialize(
        new anchor.BN(10),
        new anchor.BN(200),
        new anchor.BN(0),
        new anchor.BN(1),
        new anchor.BN(5 * 60),
        new anchor.BN(5 * 60),
        new anchor.BN(40000),
        new anchor.BN(5),
        {
          accounts: {
            ruinStakingTreasury: ruinStakingTreasury.publicKey,
            ruinStakingTerm: ruinStakingTerm.publicKey,
            ruinStaking: ruinStaking.publicKey,
            ruinStakingAdmin: deployer,
            ruinStakingToken: stakingToken,
            ruinStakingDistributor: distributorTokenAccount,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
          signers: []
        }
      )
    } catch (err) {
      assert.equal(err.message, "Signature verification failed")
      return;
    }

    assert.fail('The instruction should have failed with zero rate initialization topic.');
  });

  it("User able to stake token", async () => {
    const staking = await program.account.ruinStaking.fetch(ruinStaking.publicKey);
    const balance = await program.provider.connection.getBalance(staker.publicKey);
    console.log(balance.toString());

    await program.rpc.stake(
      new anchor.BN(100 * (10 ** 6)),
      {
        accounts: {
          ruinStakingTerm: ruinStakingTerm.publicKey,
          ruinStaking: ruinStaking.publicKey,
          investor: staker.publicKey,
          investorTokenAccount: stakerTokenAccount,
          userPendingWithdrawl,
          userStaked,
          treasuryTokenAccount: staking.treasury,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [
          staker
        ]
      }
    )

    const pendingWithdrawlResp = await program.account.pendingRewardWithdrawl.fetch(userPendingWithdrawl);
    assert.equal(pendingWithdrawlResp.pendingRewards, 0);
  });

  it("User able to harvest reward when time's passed", async () => {
    const staking = await program.account.ruinStaking.fetch(ruinStaking.publicKey);

    await program.rpc.stake(
      new anchor.BN(100 * (10 ** 6)),
      {
        accounts: {
          ruinStakingTerm: ruinStakingTerm.publicKey,
          ruinStaking: ruinStaking.publicKey,
          investor: staker.publicKey,
          investorTokenAccount: stakerTokenAccount,
          userPendingWithdrawl,
          userStaked,
          treasuryTokenAccount: staking.treasury,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [
          staker
        ]
      }
    );

    await sleep(8 * 1000);

    await program.rpc.harvest(
      {
        accounts: {
          ruinStakingTerm: ruinStakingTerm.publicKey,
          ruinStaking: ruinStaking.publicKey,
          investor: staker.publicKey,
          userPendingWithdrawl,
          userStaked,
          treasuryTokenAccount: staking.treasury,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
        signers: [
          staker
        ]
      }
    )
    const pendingWithdrawlResp = await program.account.pendingRewardWithdrawl.fetch(userPendingWithdrawl);
    const tokenBalanceBeforeClaim = await program.provider.connection.getTokenAccountBalance(stakerTokenAccount);

    await program.rpc.claimPendingReward(
      {
        accounts: {
          ruinStakingTerm: ruinStakingTerm.publicKey,
          ruinStaking: ruinStaking.publicKey,
          vaultAuthority,
          investor: staker.publicKey,
          distributorTokenAccount,
          investorTokenAccount: stakerTokenAccount,
          userPendingWithdrawl,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
        signers: [
          staker
        ]
      }
    )

    const tokenBalanceAfterClaim = await program.provider.connection.getTokenAccountBalance(stakerTokenAccount);
    assert.equal(new anchor.BN(tokenBalanceBeforeClaim.value.amount).add(pendingWithdrawlResp.pendingRewards.div(new anchor.BN(10 ** 12))).toString(), tokenBalanceAfterClaim.value.amount)
  });
});
