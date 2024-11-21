import { program } from "commander";
import { PublicKey } from "@solana/web3.js";
import {
  configProject,
  launchToken,
  setClusterConfig,
  swap,
} from "./scripts";

program.version("0.0.1");

programCommand("config").action(async (directory, cmd) => {
  const { env, keypair, rpc } = cmd.opts();

  console.log("Solana Cluster:", env);
  console.log("Keypair Path:", keypair);
  console.log("RPC URL:", rpc);

  await setClusterConfig(env, keypair, rpc);

  await configProject();
});

programCommand("launch").action(async (directory, cmd) => {
  const { env, keypair, rpc } = cmd.opts();

  console.log("Solana Cluster:", env);
  console.log("Keypair Path:", keypair);
  console.log("RPC URL:", rpc);

  await setClusterConfig(env, keypair, rpc);

  await launchToken();
});

programCommand("swap")
  .option("-t, --token <string>", "token address")
  .option("-a, --amount <number>", "swap amount")
  .option("-s, --style <string>", "0: buy token, 1: sell token")
  .action(async (directory, cmd) => {
    const { env, keypair, rpc, token, amount, style } = cmd.opts();

    console.log("Solana Cluster:", env);
    console.log("Keypair Path:", keypair);
    console.log("RPC URL:", rpc);

    await setClusterConfig(env, keypair, rpc);

    if (token === undefined) {
      console.log("Error token address");
      return;
    }

    if (amount === undefined) {
      console.log("Error swap amount");
      return;
    }

    if (style === undefined) {
      console.log("Error swap style");
      return;
    }

    await swap(token, amount, style);
  });

function programCommand(name: string) {
  return program
    .command(name)
    .option(
      //  mainnet-beta, testnet, devnet
      "-e, --env <string>",
      "Solana cluster env name",
      "devnet"
    )
    .option(
      "-r, --rpc <string>",
      "Solana cluster RPC name",
      "https://api.devnet.solana.com"
    )
    .option(
      "-k, --keypair <string>",
      "Solana wallet Keypair Path",
      "../key/uu.json"
    );
}

program.parse(process.argv);

/*

yarn script config
yarn script launch
yarn script snipe -t 4T4Tgq96SpdyATPvQ4B7yeUVWyoPCcSxF1gwAUB2z36n
yarn script close-presale -t 4T4Tgq96SpdyATPvQ4B7yeUVWyoPCcSxF1gwAUB2z36n
yarn script process-snipe -s UUQDyocFoN4tpBTp2gAWVBQ9X1tjG49Vdu6xcTu6xQv -t 4T4Tgq96SpdyATPvQ4B7yeUVWyoPCcSxF1gwAUB2z36n
yarn script swap -t 4T4Tgq96SpdyATPvQ4B7yeUVWyoPCcSxF1gwAUB2z36n -a 2000000000 -s 0


*/
