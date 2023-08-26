import * as anchor from "@coral-xyz/anchor"
import programIDL from "../target/idl/temp11.json"

let connection = new anchor.web3.Connection("http://127.0.0.1:8899",
    anchor.AnchorProvider.defaultOptions());
const airdropOnAddress = async (receiverPubKey: anchor.web3.PublicKey) => {
    let airdropSignature = await connection.requestAirdrop(
        receiverPubKey,
        anchor.web3.LAMPORTS_PER_SOL * 1000
    );
    await connection.confirmTransaction(airdropSignature);
};

(async () => {
    let providerWallet = anchor.web3.Keypair.generate();
    await airdropOnAddress(providerWallet.publicKey);
    const provider = new anchor.AnchorProvider(
        connection,
        new anchor.Wallet(providerWallet),
        anchor.AnchorProvider.defaultOptions()
    );
    anchor.setProvider(provider);
    let program = new anchor.Program(JSON.parse(JSON.stringify(programIDL)), programIDL.metadata.address, provider);

    let dataContainerAccount = anchor.web3.Keypair.generate();
    console.log(
        await program.methods.initializeDataContainer(Buffer.from("Op Bro")).accounts(
            {
                tempAccount: dataContainerAccount.publicKey,
                signer: providerWallet.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId
            }
        ).preInstructions([
            await program.account.dataContainer.createInstruction(dataContainerAccount, 60000)
        ]).signers([dataContainerAccount]).rpc()
    );
    // console.log(await program.account.dataContainer.fetch(dataContainerAccount.publicKey));
    console.log(
        await program.methods.pushInDataContainer(Buffer.from("Op Bro, Hello Bro")).accounts(
            {
                tempAccount: dataContainerAccount.publicKey,
            }
        ).rpc()
    );
    // console.log(await program.account.dataContainer.fetch(dataContainerAccount.publicKey));

    //  console.log(Uint8Array.from(program.coder.instruction.encode("pushInDataContainer", {
    //     tempData:Buffer.from("Op Bro, Hello Bro")
    //  })));
    // console.log(Uint8Array.from(["8e"]]));
    
})()

