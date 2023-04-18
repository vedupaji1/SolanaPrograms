const { expect } = require("chai");
const anchor = require("@project-serum/anchor");
const { SystemProgram } = anchor.web3;
const anchorSPL = require("@solana/spl-token");

const createProviderAndProgramInstance = async (connection) => {
  let providerKeypair = anchor.web3.Keypair.generate();
  const airdropSignature = await connection.requestAirdrop(
    providerKeypair.publicKey,
    anchor.web3.LAMPORTS_PER_SOL * 500
  );
  await connection.confirmTransaction(airdropSignature);
  let provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(providerKeypair),
    "confirmed"
  );
  anchor.setProvider(provider);

  const idl = require("../target/idl/escrow_swap.json");
  const programID = new anchor.web3.PublicKey(idl.metadata.address);
  let program = new anchor.Program(idl, programID, provider);
  return { provider, program, providerKeypair };
};

describe("temp", () => {
  const connection = new anchor.web3.Connection(
    "http://localhost:8899",
    "confirmed"
  );
  it("Is initialized!", async () => {
    let account1Data = await createProviderAndProgramInstance(connection);

    const [programStatsPublickey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [anchor.utils.bytes.utf8.encode("programStats")],
        account1Data.program.programId
      );
    console.log(
      await account1Data.program.rpc.createProgramStats({
        accounts: {
          programStats: programStatsPublickey,
          signer: account1Data.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [],
      })
    );

    const [user1AStatsPublickey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          anchor.utils.bytes.utf8.encode("userStats"),
          account1Data.provider.publicKey.toBuffer(),
        ],
        account1Data.program.programId
      );
    console.log(
      await account1Data.program.rpc.createUserStats({
        accounts: {
          userStats: user1AStatsPublickey,
          signer: account1Data.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [],
      })
    );

    let account2Data = await createProviderAndProgramInstance(connection);
    const [user2AStatsPublickey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          anchor.utils.bytes.utf8.encode("userStats"),
          account2Data.provider.publicKey.toBuffer(),
        ],
        account2Data.program.programId
      );
    console.log(
      await account2Data.program.rpc.createUserStats({
        accounts: {
          userStats: user2AStatsPublickey,
          signer: account2Data.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [],
      })
    );

    const token1 = await anchorSPL.createMint(
      connection,
      account1Data.providerKeypair,
      account1Data.provider.publicKey,
      account1Data.provider.publicKey,
      9
    );

    const token2 = await anchorSPL.createMint(
      connection,
      account2Data.providerKeypair,
      account2Data.provider.publicKey,
      account2Data.provider.publicKey,
      9
    );

    const user1TOken1Account = //
      await anchorSPL.getOrCreateAssociatedTokenAccount(
        connection,
        account1Data.providerKeypair,
        token1,
        account1Data.provider.publicKey
      );

    const user1TOken2Account =
      await anchorSPL.getOrCreateAssociatedTokenAccount(
        connection,
        account1Data.providerKeypair,
        token2,
        account1Data.provider.publicKey
      );

    const user2TOken1Account =
      await anchorSPL.getOrCreateAssociatedTokenAccount(
        connection,
        account2Data.providerKeypair,
        token1,
        account2Data.provider.publicKey
      );

    const user2TOken2Account =
      await anchorSPL.getOrCreateAssociatedTokenAccount(
        connection,
        account2Data.providerKeypair,
        token2,
        account2Data.provider.publicKey
      );

    await anchorSPL.mintTo(
      //
      connection,
      account1Data.providerKeypair,
      token1,
      user1TOken1Account.address,
      account1Data.provider.publicKey,
      100000000000
    );

    await anchorSPL.mintTo(
      connection,
      account2Data.providerKeypair,
      token2,
      user2TOken2Account.address,
      account2Data.provider.publicKey,
      100000000000
    );

    const [swapEscrowStats] = await anchor.web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("swapStats"), [0]],
      account1Data.program.programId
    );
    let token1PublicKey = new anchor.web3.PublicKey(token1.toBase58());
    let token2PublicKey = new anchor.web3.PublicKey(token2.toBase58());
    let user1TOken1AccountPublicKey = new anchor.web3.PublicKey(
      user1TOken1Account.address.toBase58()
    );
    let user1TOken2AccountPublicKey = new anchor.web3.PublicKey(
      user1TOken2Account.address.toBase58()
    );

    let user2TOken1AccountPublicKey = new anchor.web3.PublicKey(
      user2TOken1Account.address.toBase58()
    );
    let user2TOken2AccountPublicKey = new anchor.web3.PublicKey(
      user2TOken2Account.address.toBase58()
    );
    const [escrowTokenAAccount] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          anchor.utils.bytes.utf8.encode("escrowTokenAccount"),
          token1PublicKey.toBuffer(),
        ],
        account1Data.program.programId
      );

    console.log(
      await account1Data.program.rpc.createSwap(
        token2PublicKey,
        new anchor.BN(10),
        new anchor.BN(20),
        user1TOken2AccountPublicKey,
        account2Data.provider.publicKey,
        {
          accounts: {
            swapEscrowStats: swapEscrowStats,
            escrowTokenAAccount: escrowTokenAAccount,
            programStats: programStatsPublickey,
            userAStats: user1AStatsPublickey,
            userATokenAAccount: user1TOken1AccountPublicKey,
            tokenAAccount: token1PublicKey,
            signer: account1Data.provider.publicKey,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            tokenProgram: anchorSPL.TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
          signers: [],
        }
      )
    );

    console.log(
      await account2Data.program.rpc.swapTokens({
        accounts: {
          swapEscrowStats: swapEscrowStats,
          escrowTokenAAccount: escrowTokenAAccount,
          programStats: programStatsPublickey,
          userBStats: user2AStatsPublickey,
          userATokenBAccount: user1TOken2AccountPublicKey,
          userBTokenAAccount: user2TOken1AccountPublicKey,
          userBTokenBAccount: user2TOken2AccountPublicKey,
          tokenAAccount: token1PublicKey,
          tokenBAccount: token2PublicKey,
          signer: account2Data.provider.publicKey,
          tokenProgram: anchorSPL.TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        signers: [],
      })
    );

    // let token1Mint = await anchorSPL.mintTo(connection, user2TOken2Account);
    // const mintInfo = await anchorSPL.getMint(connection, token1Mint);

    // console.log(mintInfo.supply);

    // const tokenAccountInfo = await anchorSPL.getAccount(
    //   connection,
    //   user1TOken1Account.address
    // );

    // let tx2 = new anchor.web3.Transaction();
    // tx2.add(
    //   anchorSPL.createMintToCheckedInstruction(
    //     tokenAKeyPair.publicKey,
    //     userATokenAKeyPair,
    //     account1Data.provider.publicKey, // mint auth
    //     10, // amount
    //     9 // decimals
    //   )
    // );
    // console.log(
    //   `txhash: ${await connection.sendTransaction(tx2, [
    //     account1Data.providerKeypair,
    //     //account1Data.providerKeypair,
    //   ])}`
    // );

    // let userATokenAKeyPair = anchor.web3.Keypair.generate();
    // let ata = await anchorSPL.getAssociatedTokenAddress(
    //   userATokenAKeyPair.publicKey,
    //   account1Data.provider.publicKey,
    //   false
    // );

    // let tx2 = new anchor.web3.Transaction();
    // tx2.add(
    //   anchorSPL.createAssociatedTokenAccountInstruction(
    //     account1Data.provider.publicKey, // payer
    //     ata, // ata
    //     account1Data.provider.publicKey, // owner
    //     userATokenAKeyPair.publicKey // mint
    //   )
    // );
    // console.log(
    //   await connection.sendTransaction(tx2, [account1Data.providerKeypair])
    // );
    // let mainStatsAccountData = await account1Data.program.account.user.fetch(
    //   mainStatsAccount.publicKey
    // );

    // expect(mainStatsAccountData.authority.toBase58()).to.equal(
    //   account1Data.provider.wallet.publicKey.toBase58()
    // );

    // let account2Data = await createProviderAndProgramInstance(connection);

    // let mainStatsAccount2 = anchor.web3.Keypair.generate();
    // let tx2 = await account2Data.program.rpc.initialize("Op Bro, Hello Guys", {
    //   accounts: {
    //     userStats: mainStatsAccount2.publicKey,
    //     signer: account2Data.provider.wallet.publicKey,
    //     systemProgram: SystemProgram.programId,
    //   },
    //   signers: [mainStatsAccount2],
    // });
    // console.log(tx2);

    // let mainStatsAccountData2 = await account2Data.program.account.user.fetch(
    //   mainStatsAccount2.publicKey
    // );
    // expect(mainStatsAccountData2.authority.toBase58()).to.equal(
    //   account2Data.provider.wallet.publicKey.toBase58()
    // );

    // let userAccountData = await account2Data.program.account.user.all();

    // for (let i = 0; i < userAccountData.length; i++) {
    //   console.log(userAccountData[i].account.authority.toBase58());
    // }

    // await account2Data.program.rpc.changeMessage("Hello Guys", {
    //   accounts: {
    //     authority: account2Data.provider.publicKey,
    //     userStats: mainStatsAccount2.publicKey,
    //   },
    // });

    // console.log(
    //   await account2Data.program.account.user.fetch(mainStatsAccount2.publicKey)
    // );
  });
});
