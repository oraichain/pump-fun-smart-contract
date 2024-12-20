import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Pump } from "../target/types/pump";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
  ComputeBudgetProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { BN } from "bn.js";
import key1 from "./keys/user1.json";
import key2 from "./keys/user2.json";
import {
  ASSOCIATED_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@coral-xyz/anchor/dist/cjs/utils/token";
import { simulateTransaction } from "@coral-xyz/anchor/dist/cjs/utils/rpc";
import { assert, expect } from "chai";
import { execSync } from "child_process";

anchor.setProvider(anchor.AnchorProvider.env());

const connection = anchor.getProvider().connection;
const curveSeed = "CurveConfiguration";
const POOL_SEED_PREFIX = "liquidity_pool";
const LP_SEED_PREFIX = "LiqudityProvider";

describe("pump", () => {
  const program = anchor.workspace.Pump as Program<Pump>;

  // custom setting
  const user = Keypair.fromSecretKey(new Uint8Array(key1));
  const user2 = Keypair.fromSecretKey(new Uint8Array(key2));
  const tokenDecimal = 6;
  const amount = new BN(1000000000).mul(new BN(10 ** tokenDecimal));
  const airdropAmount = 10 ** 11;
  console.log(BigInt(amount.toString()));
  console.log(BigInt(amount.toString()).toString());
  console.log("🚀 ~ describe ~ amount:", amount.toString());

  let mint1: PublicKey;
  let tokenAta1: PublicKey;

  let mint2: PublicKey;
  let tokenAta2: PublicKey;

  console.log("Admin's wallet address is : ", user.publicKey.toBase58());

  it("Airdrop to admin wallet", async () => {
    console.log(
      `Requesting airdrop to admin for 1SOL : ${user.publicKey.toBase58()}`
    );
    await airdrop(user.publicKey, airdropAmount);
    await airdrop(user2.publicKey, airdropAmount);
    const adminBalance =
      (await connection.getBalance(user.publicKey)) / 10 ** 9;
    console.log("admin wallet balance : ", adminBalance, "SOL");
    assert.isAbove(adminBalance, 10);
  });

  it("Mint token1 to user wallet", async () => {
    console.log("Trying to create and mint token1 to user's wallet");

    mint1 = await createMint(
      connection,
      user,
      user.publicKey,
      user.publicKey,
      tokenDecimal
    );
    console.log("mint1 address: ", mint1.toBase58()); // 6uK5MT2i8aRGYYXrD4DyctZKwYuu91oiRd6r75SJ1kK3
    tokenAta1 = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        user,
        mint1,
        user.publicKey
      )
    ).address;
    console.log("token1 account address: ", tokenAta1.toBase58()); //G4EG27JDoyrgdS3ZJenVypePdgZMtNgDiuyYZbMJiVwm
    try {
      //minting 100 new tokens to the token address we just created
      await mintTo(
        connection,
        user,
        mint1,
        tokenAta1,
        user.publicKey,
        BigInt(amount.toString())
      );
    } catch (error) {
      console.log("🚀 ~ here:", error);
    }
    const tokenBalance = await connection.getTokenAccountBalance(tokenAta1);
    console.log("tokenBalance1 in user:", tokenBalance.value.uiAmount);
    console.log("token 1 successfully minted");
    assert.strictEqual(tokenBalance.value.uiAmount, 10 ** 9);
  });

  it("Mint token 2 to user wallet", async () => {
    console.log("Trying to create and mint token 2 to user's wallet");
    mint2 = await createMint(
      connection,
      user,
      user.publicKey,
      user.publicKey,
      tokenDecimal
    );
    console.log("mint 2 address: ", mint2.toBase58());

    tokenAta2 = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        user,
        mint2,
        user.publicKey
      )
    ).address;
    console.log("token 2 account address: ", tokenAta2.toBase58());

    await mintTo(
      connection,
      user,
      mint2,
      tokenAta2,
      user.publicKey,
      BigInt(amount.toString())
    );
    const tokenBalance = await connection.getTokenAccountBalance(tokenAta2);
    console.log("token 2 Balance in user:", tokenBalance.value.uiAmount);
    console.log("token 2 successfully minted");
    assert.equal(tokenBalance.value.uiAmount, 10 ** 9);
  });

  it("Deploy the contract", async () => {
    const res = execSync("anchor deploy --provider.wallet ./id.json");
    console.log(res.toString());
  });

  it("Initialize the contract", async () => {
    console.log("program id: ", program.programId.toBase58());
    const [curveConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from(curveSeed)],
      program.programId
    );
    const tx = new Transaction().add(
      ComputeBudgetProgram.setComputeUnitLimit({ units: 10_000 }),
      ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1200_000 }),
      await program.methods
        .initialize(1)
        .accounts({
          dexConfigurationAccount: curveConfig,
          admin: user.publicKey,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
        })
        .instruction()
    );
    tx.feePayer = user.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    // console.log(await connection.simulateTransaction(tx))
    const sig = await sendAndConfirmTransaction(connection, tx, [user], {
      skipPreflight: true,
    });
    console.log("Successfully initialized : ", sig);
    let pool = await program.account.curveConfiguration.fetch(curveConfig);
    console.log("Pool State : ", pool);
  });

  // it('create pool', async () => {
  //   try {
  //     const [poolPda] = PublicKey.findProgramAddressSync([Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer(), mint2.toBuffer()], program.programId);
  //     const [liquidityProviderAccount] = PublicKey.findProgramAddressSync([Buffer.from(LP_SEED_PREFIX), poolPda.toBuffer(), user.publicKey.toBuffer()], program.programId);
  //     const poolTokenOne = await getAssociatedTokenAddress(mint1, poolPda, true);
  //     const poolTokenTwo = await getAssociatedTokenAddress(mint2, poolPda, true);
  //     const userAta1 = await getAssociatedTokenAddress(mint1, user.publicKey);
  //     const userAta2 = await getAssociatedTokenAddress(mint2, user.publicKey);

  //     const tx = new Transaction().add(
  //       ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
  //       ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
  //       await program.methods
  //         .createPool()
  //         .accounts({
  //           pool: poolPda,
  //           mintTokenOne: mint1,
  //           mintTokenTwo: mint2,
  //           poolTokenAccountOne: poolTokenOne,
  //           poolTokenAccountTwo: poolTokenTwo,
  //           payer: user.publicKey,
  //           tokenProgram: TOKEN_PROGRAM_ID,
  //           rent: SYSVAR_RENT_PUBKEY,
  //           associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
  //           systemProgram: SystemProgram.programId
  //         })
  //         .instruction()
  //     );
  //     tx.feePayer = user.publicKey;
  //     tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  //     // console.log(await connection.simulateTransaction(tx))
  //     const sig = await sendAndConfirmTransaction(connection, tx, [user], { skipPreflight: true });
  //     console.log('Successfully created pool : ', sig);
  //   } catch (error) {
  //     console.log('Error in creating pool', error);
  //   }
  // });

  it("add liquidity", async () => {
    try {
      const [poolPda] = PublicKey.findProgramAddressSync(
        [Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer()],
        program.programId
      );

      const [liquidityProviderAccount] = PublicKey.findProgramAddressSync(
        [
          Buffer.from(LP_SEED_PREFIX),
          poolPda.toBuffer(),
          user.publicKey.toBuffer(),
        ],
        program.programId
      );
      const poolTokenOne = await getAssociatedTokenAddress(
        mint1,
        poolPda,
        true
      );
      const userAta1 = await getAssociatedTokenAddress(mint1, user.publicKey);

      const tx = new Transaction().add(
        ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
        await program.methods
          .addLiquidity(new BN(1000000000000000), new BN(30000000000))
          .accounts({
            pool: poolPda,
            mintTokenOne: mint1,
            poolTokenAccountOne: poolTokenOne,
            userTokenAccountOne: userAta1,
            liquidityProviderAccount: liquidityProviderAccount,
            user: user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
          })
          .instruction()
      );
      tx.feePayer = user.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      // console.log(await connection.simulateTransaction(tx))
      const sig = await sendAndConfirmTransaction(connection, tx, [user], {
        skipPreflight: true,
      });
      console.log("Successfully added liquidity : ", sig);

      const signature = await connection.requestAirdrop(poolPda, 10 ** 9);
      // 2 - Fetch the latest blockhash
      const { blockhash, lastValidBlockHeight } =
        await connection.getLatestBlockhash();
      // 3 - Confirm transaction success
      await connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature,
      });
    } catch (error) {
      console.log("Error in adding liquidity", error);
    }
  });

  it("Swap token", async () => {
    try {
      const [curveConfig] = PublicKey.findProgramAddressSync(
        [Buffer.from(curveSeed)],
        program.programId
      );
      const [poolPda] = PublicKey.findProgramAddressSync(
        [Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer()],
        program.programId
      );
      const poolTokenOne = await getAssociatedTokenAddress(
        mint1,
        poolPda,
        true
      );
      const userAta1 = await getAssociatedTokenAddress(mint1, user.publicKey);

      console.log(
        "pooltoken one",
        poolTokenOne.toBase58(),
        "program id",
        program.programId.toBase58(),
        "userAta1",
        userAta1.toBase58()
      );

      const tx = new Transaction().add(
        ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
        await program.methods
          .swap(new BN(200000000), new BN(2))
          .accounts({
            pool: poolPda,
            mintTokenOne: mint1,
            poolTokenAccountOne: poolTokenOne,
            userTokenAccountOne: userAta1,
            dexConfigurationAccount: curveConfig,
            user: user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
          })
          .instruction()
      );
      tx.feePayer = user.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      console.log("recentBlockhash", tx.recentBlockhash);
      console.log("simulate", await simulateTransaction(connection, tx));
      const sig = await sendAndConfirmTransaction(connection, tx, [user], {
        skipPreflight: true,
      });

      console.log("Successfully swapped : ", sig);
    } catch (error) {
      console.log("Error in swap transaction", error);
    }
  });

  it("Remove liquidity", async () => {
    try {
      const [poolPda] = PublicKey.findProgramAddressSync(
        [Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer()],
        program.programId
      );
      const [liquidityProviderAccount] = PublicKey.findProgramAddressSync(
        [
          Buffer.from(LP_SEED_PREFIX),
          poolPda.toBuffer(),
          user.publicKey.toBuffer(),
        ],
        program.programId
      );
      const poolTokenOne = await getAssociatedTokenAddress(
        mint1,
        poolPda,
        true
      );
      const userAta1 = await getAssociatedTokenAddress(mint1, user.publicKey);
      const tx = new Transaction().add(
        ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
        await program.methods
          .removeLiquidity(new BN(10))
          .accounts({
            pool: poolPda,
            mintTokenOne: mint1,
            poolTokenAccountOne: poolTokenOne,
            userTokenAccountOne: userAta1,
            liquidityProviderAccount: liquidityProviderAccount,
            user: user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
          })
          .instruction()
      );
      tx.feePayer = user.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      console.log(await connection.simulateTransaction(tx));
      const sig = await sendAndConfirmTransaction(connection, tx, [user], {
        skipPreflight: true,
      });
      console.log("Successfully Removed liquidity : ", sig);
    } catch (error) {
      console.log("Error in removing liquidity", error);
    }
  });
});

function comparePublicKeys(pubkey1: PublicKey, pubkey2: PublicKey): number {
  const key1Bytes = pubkey1.toBuffer();
  const key2Bytes = pubkey2.toBuffer();

  for (let i = 0; i < key1Bytes.length; i++) {
    if (key1Bytes[i] > key2Bytes[i]) {
      return 1;
    } else if (key1Bytes[i] < key2Bytes[i]) {
      return -1;
    }
  }
  return 0;
}

function generateSeed(tokenOne: PublicKey, tokenTwo: PublicKey): string {
  return comparePublicKeys(tokenOne, tokenTwo) > 0
    ? `${tokenOne.toString()}${tokenTwo.toString()}`
    : `${tokenTwo.toString()}${tokenOne.toString()}`;
}

async function airdrop(publicKey: PublicKey, amount: number) {
  // 1 - Request Airdrop
  const signature = await connection.requestAirdrop(publicKey, amount);
  // 2 - Fetch the latest blockhash
  const { blockhash, lastValidBlockHeight } =
    await connection.getLatestBlockhash();
  // 3 - Confirm transaction success
  await connection.confirmTransaction({
    blockhash,
    lastValidBlockHeight,
    signature,
  });
}
