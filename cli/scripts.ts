import * as anchor from "@coral-xyz/anchor";
import { BN, Program, Provider, web3 } from '@coral-xyz/anchor';
import fs from 'fs';

import {
    Keypair,
    Connection,
    PublicKey,
} from '@solana/web3.js';

import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';

import { Pumpfun } from "../target/types/pumpfun";
import { createConfigTx, launchTokenTx, swapTx } from '../lib/scripts';
import { execTx } from '../lib/util';
import { PRESALE_TIME, TEST_DECIMALS, TEST_NAME, TEST_SYMBOL, TEST_TOKEN_SUPPLY, TEST_URI, TEST_VIRTUAL_RESERVES } from "../lib/constant";

let solConnection: Connection = null;
let program: Program<Pumpfun> = null;
let payer: NodeWallet = null;

/**
 * Set cluster, provider, program
 * If rpc != null use rpc, otherwise use cluster param
 * @param cluster - cluster ex. mainnet-beta, devnet ...
 * @param keypair - wallet keypair
 * @param rpc - rpc
 */
export const setClusterConfig = async (
    cluster: web3.Cluster,
    keypair: string,
    rpc?: string
) => {

    if (!rpc) {
        solConnection = new web3.Connection(web3.clusterApiUrl(cluster));
    } else {
        solConnection = new web3.Connection(rpc);
    }

    const walletKeypair = Keypair.fromSecretKey(
        Uint8Array.from(JSON.parse(fs.readFileSync(keypair, 'utf-8'))),
        { skipValidation: true });
    payer = new NodeWallet(walletKeypair);

    console.log('Wallet Address: ', payer.publicKey.toBase58());

    anchor.setProvider(new anchor.AnchorProvider(
        solConnection,
        payer,
        { skipPreflight: true, commitment: 'confirmed' }));

    // Generate the program client from IDL.
    program = anchor.workspace.Pumpfun as Program<Pumpfun>;

    console.log('ProgramId: ', program.programId.toBase58());
}

export const configProject = async () => {
    // Create a dummy config object to pass as argument.
    const newConfig = {
      authority: payer.publicKey,
      pendingAuthority: PublicKey.default,
  
      platformBuyFee: 5.0, // Example fee: 5%
      platformSellFee: 5.0, // Example fee: 5%
  
      platformMigrateFeeBps: 500, //  Example fee: 5%
  
      presaleTime: new BN(PRESALE_TIME), // Example time: 2 secs
      curveLimit: new BN(4_000_000_000), //  Example limit: 2 SOL
  
      lamportAmountConfig: { range: { min: new BN(1000000000), max: new BN(100000000000) } },
      tokenSupplyConfig: { range: { min: new BN(5000), max: new BN(2000000) } },
      tokenDecimalsConfig: { range: { min: 6, max: 9 } },
    };

    const tx = await createConfigTx(payer.publicKey, newConfig, solConnection, program);

    await execTx(tx, solConnection, payer);
}

export const launchToken = async () => {
    const tx = await launchTokenTx(
        TEST_DECIMALS,
        TEST_TOKEN_SUPPLY,
        TEST_VIRTUAL_RESERVES,
        
        //  metadata
        TEST_NAME,
        TEST_SYMBOL,
        TEST_URI,
        
        payer.publicKey,
        
        solConnection,
        program
    );

    await execTx(tx, solConnection, payer);
}

export const swap = async (
    token: PublicKey,

    amount: number,
    style: number
) => {
    const tx = await swapTx(payer.publicKey, token, amount, style, solConnection, program);

    await execTx(tx, solConnection, payer);
}
