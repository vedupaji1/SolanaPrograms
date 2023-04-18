const anchor = require("@project-serum/anchor");
const anchorSPL = require("@solana/spl-token");
describe("escrowSwap", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  it("Is initialized!", async () => {
    // Add your test here.
    const program = anchor.workspace.EscrowSwap;
    let tokenKeyPair = anchor.web3.Keypair.generate();
    await program.rpc.createSplToken({
      accounts: {
        mint: tokenKeyPair.publicKey,
        signer: provider.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: anchorSPL.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [tokenKeyPair],
    });

    let userTokenAccountKeyPair = anchor.web3.Keypair.generate();
    await program.rpc.createUserSplTokenAccount({
      accounts: {
        userSplAccount: userTokenAccountKeyPair.publicKey,
        mint: tokenKeyPair.publicKey,
        signer: provider.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: anchorSPL.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [userTokenAccountKeyPair],
    });

    await program.rpc.mintToken({
      accounts: {
        tokenAccount: tokenKeyPair.publicKey,
        userTokenAccount: userTokenAccountKeyPair.publicKey,
        signer: provider.publicKey,
        tokenProgram: anchorSPL.TOKEN_PROGRAM_ID,
      },
      signers: [],
    });
  });
});
