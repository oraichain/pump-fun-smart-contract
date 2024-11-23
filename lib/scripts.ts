import { BN, Program } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
} from "@solana/web3.js";

import { Pumpfun } from "../target/types/pumpfun";
import { SEED_CONFIG } from "./constant";

export const createConfigTx = async (
  admin: PublicKey,

  newConfig: any,
  
  connection: Connection,
  program: Program<Pumpfun>
) => {

  const tx = await program.methods
    .configure(newConfig)
    .accounts({
      payer: admin
    })
    .transaction();

  tx.feePayer = admin;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

  return tx;
};

export const launchTokenTx = async (
  decimal: number,
  supply: number,
  reserve: number,
  name: string,
  symbol: string,
  uri: string,

  user: PublicKey,

  connection: Connection,
  program: Program<Pumpfun>
) => {
  const tokenKp = Keypair.generate();

  console.log("token address: ", tokenKp.publicKey.toBase58());

  // Send the transaction to launch a token
  const tx = await program.methods
    .launch(
      //  launch config
      decimal,
      new BN(supply),
      new BN(reserve),

      //  metadata
      name,
      symbol,
      uri
    )
    .accounts({
      creator: user,
      token: tokenKp.publicKey
    })
    .transaction();

  tx.feePayer = user;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

  tx.sign(tokenKp);

  return tx;
}

export const swapTx = async (
  user: PublicKey,
  token: PublicKey,

  amount: number,
  style: number,

  connection: Connection,
  program: Program<Pumpfun>
) => {
  const [configPda, _] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_CONFIG)],
    program.programId
  );
  const configAccount = await program.account.config.fetch(configPda);
  
  const tx = await program.methods
    .swap(new BN(amount), style, new BN(amount))
    .accounts({
      teamWallet: configAccount.teamWallet,
      
      user,
      tokenMint: token
    })
    .transaction();

  tx.feePayer = user;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

  return tx;
}
