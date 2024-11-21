
import {
    AddressLookupTableAccount,
    TransactionInstruction,
    VersionedTransaction,
    Transaction,
    Connection,
    SystemProgram,
    SYSVAR_RENT_PUBKEY
} from "@solana/web3.js";

import {
    PublicKey,
} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";

export const getAssociatedTokenAccount = (
    ownerPubkey: PublicKey,
    mintPk: PublicKey
): PublicKey => {
    let associatedTokenAccountPubkey = (PublicKey.findProgramAddressSync(
        [
            ownerPubkey.toBytes(),
            TOKEN_PROGRAM_ID.toBytes(),
            mintPk.toBytes(), // mint address
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    ))[0];

    return associatedTokenAccountPubkey;
}

export const getAdressLookupTableAccounts = async (
    keys: string[],
    connection: Connection
): Promise<AddressLookupTableAccount[]> => {
    const addressLookupTableAccountInfos =
        await connection.getMultipleAccountsInfo(
        keys.map((key) => new PublicKey(key))
        );

    return addressLookupTableAccountInfos.reduce((acc, accountInfo, index) => {
        const addressLookupTableAddress = keys[index];
        if (accountInfo) {
        const addressLookupTableAccount = new AddressLookupTableAccount({
            key: new PublicKey(addressLookupTableAddress),
            state: AddressLookupTableAccount.deserialize(accountInfo.data),
        });
        acc.push(addressLookupTableAccount);
        }

        return acc;
    }, new Array<AddressLookupTableAccount>());
};
  
export const instructionDataToTransactionInstruction = (
    instructionPayload: any
) => {
    if (instructionPayload === null) {
        return null;
    }

    return new TransactionInstruction({
        programId: new PublicKey(instructionPayload.programId),
        keys: instructionPayload.accounts.map((key) => ({
        pubkey: new PublicKey(key.pubkey),
        isSigner: key.isSigner,
        isWritable: key.isWritable,
        })),
        data: Buffer.from(instructionPayload.data, "base64"),
    });
};

export const execTx = async (
    transaction: Transaction | VersionedTransaction,
    connection: Connection,
    payer: NodeWallet
) => {
    try {
        //  Sign the transaction with payer wallet
        const signedTx = await payer.signTransaction(transaction);

        // Serialize, send and confirm the transaction
        const rawTransaction = signedTx.serialize()

        const txid = await connection.sendRawTransaction(rawTransaction, {
            skipPreflight: true,
            maxRetries: 2,
            preflightCommitment: "processed"
        });
        console.log(`https://solscan.io/tx/${txid}?cluster=custom&customUrl=${connection.rpcEndpoint}`);

        const confirmed = await connection.confirmTransaction(txid, "confirmed");

        console.log("err ", confirmed.value.err)
    } catch (e) {
        console.log(e);
    }
}


export const createAssociatedTokenAccountInstruction = (
    associatedTokenAddress: PublicKey,
    payer: PublicKey,
    walletAddress: PublicKey,
    splTokenMintAddress: PublicKey
) => {
    const keys = [
        { pubkey: payer, isSigner: true, isWritable: true },
        { pubkey: associatedTokenAddress, isSigner: false, isWritable: true },
        { pubkey: walletAddress, isSigner: false, isWritable: false },
        { pubkey: splTokenMintAddress, isSigner: false, isWritable: false },
        {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
        },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        {
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
        },
    ];
    return new TransactionInstruction({
        keys,
        programId: ASSOCIATED_TOKEN_PROGRAM_ID,
        data: Buffer.from([]),
    });
};

export const getATokenAccountsNeedCreate = async (
    connection: Connection,
    walletAddress: PublicKey,
    owner: PublicKey,
    nfts: PublicKey[],
) => {
    const instructions = []; const destinationAccounts = [];
    for (const mint of nfts) {
        const destinationPubkey = getAssociatedTokenAccount(owner, mint);
        let response = await connection.getAccountInfo(destinationPubkey);
        if (!response) {
            const createATAIx = createAssociatedTokenAccountInstruction(
                destinationPubkey,
                walletAddress,
                owner,
                mint,
            );
            instructions.push(createATAIx);
        }
        destinationAccounts.push(destinationPubkey);
        if (walletAddress != owner) {
            const userAccount = getAssociatedTokenAccount(walletAddress, mint);
            response = await connection.getAccountInfo(userAccount);
            if (!response) {
                const createATAIx = createAssociatedTokenAccountInstruction(
                    userAccount,
                    walletAddress,
                    walletAddress,
                    mint,
                );
                instructions.push(createATAIx);
            }
        }
    }
    return {
        instructions,
        destinationAccounts,
    };
};

export function calculateTotalAccountSize(
    individualAccountSize: number,
    accountHeaderSize: number,
    length: number
) {
    const accountPadding = 12;
    const minRequiredSize =
        accountPadding + accountHeaderSize + length * individualAccountSize;

    const modulo = minRequiredSize % 8;

    return modulo <= 4
        ? minRequiredSize + (4 - modulo)
        : minRequiredSize + (8 - modulo + 4);
}

const EVENT_QUEUE_LENGTH = 2978;
const EVENT_SIZE = 88;
const EVENT_QUEUE_HEADER_SIZE = 32;

const REQUEST_QUEUE_LENGTH = 63;
const REQUEST_SIZE = 80;
const REQUEST_QUEUE_HEADER_SIZE = 32;

const ORDERBOOK_LENGTH = 909;
const ORDERBOOK_NODE_SIZE = 72;
const ORDERBOOK_HEADER_SIZE = 40;

export const TOTAL_EVENT_QUEUE_SIZE = calculateTotalAccountSize(
    128,
    EVENT_QUEUE_HEADER_SIZE,
    EVENT_SIZE
)

export const TOTAL_REQUEST_QUEUE_SIZE = calculateTotalAccountSize(
    10,
    REQUEST_QUEUE_HEADER_SIZE,
    REQUEST_SIZE
)

export const TOTAL_ORDER_BOOK_SIZE = calculateTotalAccountSize(
    201,
    ORDERBOOK_HEADER_SIZE,
    ORDERBOOK_NODE_SIZE
)
