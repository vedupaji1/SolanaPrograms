import * as anchor from "@coral-xyz/anchor";
import { createAccount, createAssociatedTokenAccount } from "@solana/spl-token";
import {
  createCreateMetadataAccountV3Instruction,
  DataV2,
} from "@metaplex-foundation/mpl-token-metadata";
import { findMetadataPda } from "@metaplex-foundation/js";
import userWalletKeyPairBuffer from "../../../id.json";
import tempUserWalletKeyPairBuffer from "../../temp/wallets/user2.json";

const connection = new anchor.web3.Connection(
  // "https://solana-devnet.g.alchemy.com/v2/vKtzany_S6S8LxQb4Fen9jsxOAOdKjvj",
  "http://127.0.0.1:8899",
  anchor.AnchorProvider.defaultOptions()
);
let userWallet = anchor.web3.Keypair.fromSecretKey(
  Uint8Array.from(userWalletKeyPairBuffer)
);
let tempUserWallet = anchor.web3.Keypair.fromSecretKey(
  Uint8Array.from(userWalletKeyPairBuffer)
);

(async () => {
  console.log(await connection.getBalance(userWallet.publicKey));
  let tokenPubKey = new anchor.web3.PublicKey(
    "441HL6BxmGruaK2VkvPjpxDjrHkUwaZVzwZKBSwJ5USf"
  );
  let tokenMetaDataPDA = findMetadataPda(tokenPubKey);
  // To Create MetaData Account For Mint Account Or Token
  let transactionData = new anchor.web3.Transaction().add(
    createCreateMetadataAccountV3Instruction(
      {
        metadata: tokenMetaDataPDA,
        mint: tokenPubKey,
        mintAuthority: userWallet.publicKey,
        payer: userWallet.publicKey,
        updateAuthority: userWallet.publicKey,
      },
      {
        createMetadataAccountArgsV3: {
          data: {
            name: "EtherPepe",
            symbol: "ETHPEPE",
            uri: "https://static.news.bitcoin.com/wp-content/uploads/2023/04/pepes.jpg",
            sellerFeeBasisPoints: 0,
            creators: null,
            collection: null,
            uses: null,
          } as DataV2,
          isMutable: true,
          collectionDetails: null,
        },
      }
    )
  );

  // // To Create Associated Token Account
  // console.log(
  //     await createAssociatedTokenAccount(
  //         connection,
  //         userWallet,
  //         tokenPubKey,
  //         userWallet.publicKey,// User Or Owner Must Be Signer
  //         anchor.AnchorProvider.defaultOptions()
  //     )
  // );

  // // To Create TokenAccount
  // let tempUserTokenAccountKeyPair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from([
  //     89, 2, 167, 12, 222, 208, 139, 3, 78, 86, 245,
  //     55, 114, 108, 73, 125, 242, 130, 121, 6, 204, 174,
  //     48, 80, 63, 176, 15, 125, 146, 7, 114, 63, 208,
  //     111, 21, 236, 33, 50, 79, 144, 160, 204, 92, 143,
  //     141, 99, 82, 69, 7, 111, 16, 137, 95, 247, 27,
  //     9, 78, 84, 158, 78, 18, 124, 179, 90
  // ]))

  // console.log(tempUserTokenAccountKeyPair.publicKey);
  // console.log(await createAccount(connection, userWallet, tokenPubKey, new anchor.web3.PublicKey(
  //     "GuErscWC5pYEjBND9sPjUXAdvFXt4Xgxtsuce11PiDG3"
  // ), tempUserTokenAccountKeyPair));

  // let transactionData=new anchor.web3.Transaction(
  //    createAssociatedTokenAccount(),
  // ).add(

  // );
  // console.log(await anchor.web3.sendAndConfirmTransaction(connection, transactionData, [userWallet]));
})();
