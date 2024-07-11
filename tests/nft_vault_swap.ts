import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftVaultSwap } from "../target/types/nft_vault_swap";
import { expect } from "chai";

describe("nft_vault_swap", () => {
  // Configure the client to use the local cluster.
  
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.NftVaultSwap as Program<NftVaultSwap>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Initializes the vault", async () => {
    const [vaultPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("vault")],
      program.programId
    );

    const tx = await program.methods.initializeVault().accounts({
      authority:
    })
  });
});
