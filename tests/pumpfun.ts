import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { Pumpfun } from "../target/types/pumpfun";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import * as assert from "assert";
import { SEED_CONFIG, SEED_BONDING_CURVE, TEST_DECIMALS, TEST_NAME, TEST_SYMBOL, TEST_TOKEN_SUPPLY, TEST_URI, TEST_VIRTUAL_RESERVES } from "./constant";
import { getAssociatedTokenAccount, sleep } from "./utils";
import { token } from "@coral-xyz/anchor/dist/cjs/utils";
require("dotenv").config();

describe("pumpfun", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Pumpfun as Program<Pumpfun>;

  const adminKp = Keypair.generate();
  const userKp = Keypair.generate();
  const user2Kp = Keypair.generate();
  const tokenKp = Keypair.generate();

  console.log("admin: ", adminKp.publicKey.toBase58());
  console.log("user: ", userKp.publicKey.toBase58());
  console.log("user2: ", user2Kp.publicKey.toBase58());

  const connection = provider.connection;

  before(async () => {
    console.log("airdrop SOL to admin")

    const airdropTx = await connection.requestAirdrop(
      adminKp.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropTx);

    console.log("airdrop SOL to user")
    const airdropTx2 = await connection.requestAirdrop(
      userKp.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropTx2);

    console.log("airdrop SOL to user2")
    const airdropTx3 = await connection.requestAirdrop(
      user2Kp.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropTx3);
  });

  it("Is configured", async () => {

    // Create a dummy config object to pass as argument.
    const newConfig = {
      authority: adminKp.publicKey,
      pendingAuthority: PublicKey.default,

      teamWallet: adminKp.publicKey,

      platformBuyFee: 5.0, // Example fee: 5%
      platformSellFee: 5.0, // Example fee: 5%

      curveLimit: new BN(4_000_000_000), //  Example limit: 2 SOL

      lamportAmountConfig: { range: { min: new BN(1000000000), max: new BN(100000000000) } },
      tokenSupplyConfig: { range: { min: new BN(5000), max: new BN(2000000) } },
      tokenDecimalsConfig: { range: { min: 6, max: 9 } },
    };

    // Send the transaction to configure the program.
    const tx = await program.methods
      .configure(newConfig)
      .accounts({
        payer: adminKp.publicKey
      })
      .signers([adminKp])
      .rpc();

    console.log("tx signature:", tx);

    // get PDA for the config account using the seed "config".
    const [configPda, _] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_CONFIG)],
      program.programId
    );

    // Log PDA details for debugging.
    console.log("config PDA:", configPda.toString());

    // Fetch the updated config account to validate the changes.
    const configAccount = await program.account.config.fetch(configPda);

    // Assertions to verify configuration
    assert.equal(configAccount.authority.toString(), adminKp.publicKey.toString());
    assert.equal(configAccount.platformBuyFee, 5);
    assert.equal(configAccount.platformSellFee, 5);
    assert.equal(parseFloat(configAccount.lamportAmountConfig.range.min.toString()), 1000000000);
    assert.equal(parseFloat(configAccount.lamportAmountConfig.range.max.toString()), 100000000000);
    assert.equal(parseFloat(configAccount.tokenSupplyConfig.range.min.toString()), 5000);
    assert.equal(parseFloat(configAccount.tokenSupplyConfig.range.max.toString()), 2000000);
    assert.equal(parseFloat(configAccount.tokenDecimalsConfig.range.min.toString()), 6);
    assert.equal(parseFloat(configAccount.tokenDecimalsConfig.range.max.toString()), 9);
  });

  it("Is the token created", async () => {

    console.log("token: ", tokenKp.publicKey.toBase58());

    // Send the transaction to launch a token
    const tx = await program.methods
      .launch(
        //  launch config
        TEST_DECIMALS,
        new BN(TEST_TOKEN_SUPPLY),
        new BN(TEST_VIRTUAL_RESERVES),
        
        //  metadata
        TEST_NAME,
        TEST_SYMBOL,
        TEST_URI
      )
      .accounts({
        creator: userKp.publicKey,
        token: tokenKp.publicKey
      })
      .signers([userKp, tokenKp])
      .rpc();

    console.log("tx signature:", tx);

    // get token detailed info
    const supply = await connection.getTokenSupply(tokenKp.publicKey);

    // Assertions to verify configuration
    assert.equal(supply.value.amount, TEST_TOKEN_SUPPLY);

    
    // check launch phase is 'Presale'
    const [bondingCurvePda, _] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_BONDING_CURVE), tokenKp.publicKey.toBytes()],
      program.programId
    );

    console.log("bonding curve PDA:", bondingCurvePda.toString());

    const curveAccount = await program.account.bondingCurve.fetch(bondingCurvePda);

    // Assertions to verify configuration
    assert.equal(curveAccount.creator.toBase58(), userKp.publicKey.toBase58());

  });

  it("Is user1's swap SOL for token completed", async () => {

    const [configPda, _] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_CONFIG)],
      program.programId
    );
    const configAccount = await program.account.config.fetch(configPda);

    // Send the transaction to launch a token
    const tx = await program.methods
      .swap(new BN(5_000_000), 0)
      .accounts({
        teamWallet: configAccount.teamWallet,
        user: userKp.publicKey,
        tokenMint: tokenKp.publicKey,
      })
      .signers([userKp])
      .rpc();

    console.log("tx signature:", tx);

    //  check user1's balance
    const tokenAccount = getAssociatedTokenAccount(userKp.publicKey, tokenKp.publicKey);
    const balance = await connection.getBalance(userKp.publicKey);
    const tokenBalance = await connection.getTokenAccountBalance(tokenAccount);

    console.log("buyer: ", userKp.publicKey.toBase58());
    console.log("lamports: ", balance);
    console.log("token amount: ", tokenBalance.value.uiAmount);

  });

  it("Is user1's swap Token for SOL completed", async () => {

    const [configPda, _] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_CONFIG)],
      program.programId
    );
    const configAccount = await program.account.config.fetch(configPda);

    // Send the transaction to launch a token
    const tx = await program.methods
      .swap(new BN(23_000_000), 1)
      .accounts({
        teamWallet: configAccount.teamWallet,
        user: userKp.publicKey,
        tokenMint: tokenKp.publicKey,
      })
      .signers([userKp])
      .rpc();

    console.log("tx signature:", tx);

    //  check user1's balance
    const tokenAccount = getAssociatedTokenAccount(userKp.publicKey, tokenKp.publicKey);
    const balance = await connection.getBalance(userKp.publicKey);
    const tokenBalance = await connection.getTokenAccountBalance(tokenAccount);

    console.log("buyer: ", userKp.publicKey.toBase58());
    console.log("lamports: ", balance);
    console.log("token amount: ", tokenBalance.value.uiAmount);
  });

  it("Is the curve reached the limit", async () => {

    const [configPda, _] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_CONFIG)],
      program.programId
    );
    const configAccount = await program.account.config.fetch(configPda);

    // Send the transaction to launch a token
    const tx = await program.methods
      .swap(new BN(3_000_000_000), 0)
      .accounts({
        teamWallet: configAccount.teamWallet,
        user: user2Kp.publicKey,
        tokenMint: tokenKp.publicKey,
      })
      .signers([user2Kp])
      .rpc();

    console.log("tx signature:", tx);


    //  check user2's balance
    const tokenAccount = getAssociatedTokenAccount(user2Kp.publicKey, tokenKp.publicKey);
    const balance = await connection.getBalance(user2Kp.publicKey);
    const tokenBalance = await connection.getTokenAccountBalance(tokenAccount);

    console.log("buyer: ", user2Kp.publicKey.toBase58());
    console.log("lamports: ", balance);
    console.log("token amount: ", tokenBalance.value.uiAmount);


    // check launch phase is 'completed'
    const [bondingCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED_BONDING_CURVE), tokenKp.publicKey.toBytes()],
      program.programId
    );

    const curveAccount = await program.account.bondingCurve.fetch(bondingCurvePda);

    // Assertions to verify configuration
    assert.equal(curveAccount.isCompleted, true);

  });
});
