const { expect } = require("chai");
const anchor = require("@project-serum/anchor");
const { SystemProgram } = anchor.web3;

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

  const idl = require("../target/idl/temp.json");
  const programID = new anchor.web3.PublicKey(idl.metadata.address);
  let program = new anchor.Program(idl, programID, provider);
  return { provider, program };
};

describe("temp", () => {
  const connection = new anchor.web3.Connection(
    "http://localhost:8899",
    "confirmed"
  );
  it("Is initialized!", async () => {
    let account1Data = await createProviderAndProgramInstance(connection);

    let mainStatsAccount = anchor.web3.Keypair.generate();
    let tx = await account1Data.program.rpc.initialize("Op Bro", {
      accounts: {
        userStats: mainStatsAccount.publicKey,
        signer: account1Data.provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [mainStatsAccount],
    });
    console.log(tx);

    let mainStatsAccountData = await account1Data.program.account.user.fetch(
      mainStatsAccount.publicKey
    );

    expect(mainStatsAccountData.authority.toBase58()).to.equal(
      account1Data.provider.wallet.publicKey.toBase58()
    );

    let account2Data = await createProviderAndProgramInstance(connection);

    let mainStatsAccount2 = anchor.web3.Keypair.generate();
    let tx2 = await account2Data.program.rpc.initialize("Op Bro, Hello Guys", {
      accounts: {
        userStats: mainStatsAccount2.publicKey,
        signer: account2Data.provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [mainStatsAccount2],
    });
    console.log(tx2);

    let mainStatsAccountData2 = await account2Data.program.account.user.fetch(
      mainStatsAccount2.publicKey
    );
    expect(mainStatsAccountData2.authority.toBase58()).to.equal(
      account2Data.provider.wallet.publicKey.toBase58()
    );

    let userAccountData = await account2Data.program.account.user.all();

    for (let i = 0; i < userAccountData.length; i++) {
      console.log(userAccountData[i].account.authority.toBase58());
    }

    await account2Data.program.rpc.changeMessage("Hello Guys", {
      accounts: {
        authority: account2Data.provider.publicKey,
        userStats: mainStatsAccount2.publicKey,
      },
    });

    console.log(
      await account2Data.program.account.user.fetch(mainStatsAccount2.publicKey)
    );
    // console.log(mainStatsAccountData.data.toString());
    // console.log(provider.publicKey.toBase58());
  });
});
