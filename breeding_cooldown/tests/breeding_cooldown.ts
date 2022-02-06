import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { BreedingCooldown } from '../target/types/breeding_cooldown';
import { ASSOCIATED_TOKEN_PROGRAM_ID, MintLayout, Token } from '@solana/spl-token';
import { createMint, createTokenAccount, getTokenAccount } from '@project-serum/common';
import { createAssociatedTokenAccountInstruction, mintToAccount } from './utils';
import { PublicKey } from '@solana/web3.js';
import { TokenInstructions } from '@project-serum/serum';
import { v4 as uuidv4 } from 'uuid';

const { SystemProgram, SYSVAR_RENT_PUBKEY } = anchor.web3;
const assert = require("assert");

describe('breeding_cooldown', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  // TODO: remove this constant once @project-serum/serum uses the same version
  //       of @solana/web3.js as anchor (or switch packages).
  const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
    TokenInstructions.TOKEN_PROGRAM_ID.toString()
  );

  const connection = anchor.getProvider().connection;
  const program = (<any>anchor).workspace.BreedingCooldown as Program<BreedingCooldown>;
  const PREFIX = Buffer.from(anchor.utils.bytes.utf8.encode('bapeBrd2'));
  const PREFIX_POTION = Buffer.from(anchor.utils.bytes.utf8.encode('potion'));
  const PREFIX_COUNT = Buffer.from(anchor.utils.bytes.utf8.encode('count'));
  const PREFIX_URI = Buffer.from(anchor.utils.bytes.utf8.encode('uri'));
  const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
  );

  const wallet = anchor.getProvider().wallet;
  const userPubKey = wallet.publicKey;

  const urisAccount = anchor.web3.Keypair.generate();
  const urisKey = urisAccount.publicKey;
  console.log('URIs pubKey: ' + urisKey)

  it('Initializes a vector of URIs',async () => {
      await program.rpc.initUris({
        accounts: {
          user: userPubKey,
          uris: urisKey,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY
        },
        signers: [urisAccount],
        preInstructions: [
          await createUrisAccount(urisKey)
        ]
      })
  })

  it('Adds URIs to vector',async () => {
    // with 10k bytes, got to 248 strings. 50k bytes is too big
    // using Vec<u8> got same thing... can we compress bytes to an int?
    let NUM_URIS = 10; 
    let expectedUris = [];
    for (let i = 0; i < NUM_URIS; i++) {
      console.log(i);
      let relativeUri = uuidv4()
      expectedUris.push(relativeUri)
      await program.rpc.addUri(i, relativeUri, {
        accounts: {
          user: userPubKey,
          uris: urisKey,
          systemProgram: SystemProgram.programId
        }
      })
    }

    for (let i = 0; i < NUM_URIS; i++) {
      console.log('testing ' + i);
      let expectedUri = expectedUris[i];
      const deserialized = anchor.web3.Keypair.generate();
      await program.rpc.deserializeUri(i, {
        accounts: {
          user: userPubKey,
          uris: urisKey,
          deserialized: deserialized.publicKey,
          systemProgram: SystemProgram.programId
        },
        signers: [deserialized]
      })

      let deserializedUri = await program.account.deserializedUri.fetch(deserialized.publicKey);
      assert(deserializedUri.relativeUri.toString() == expectedUri.toString());
    }

})

const CONFIG_ARRAY_START: number = 8; // key
const MAX_URI_LENGTH: number = 50;
const CONFIG_LINE_SIZE: number = MAX_URI_LENGTH; // 4 + MAX_URI_LENGTH;

async function createUrisAccount(
  urisAccount: PublicKey,
  itemsAvailable: number = 3333,
) {
  const size =
    CONFIG_ARRAY_START +
    4 +
    itemsAvailable * CONFIG_LINE_SIZE +
    8; // +
    // 2 * (Math.floor(itemsAvailable / 8) + 1);

  return anchor.web3.SystemProgram.createAccount({
    fromPubkey: userPubKey,
    newAccountPubkey: urisAccount,
    space: size,
    lamports:
      await program.provider.connection.getMinimumBalanceForRentExemption(
        size,
      ),
    programId: program.programId,
  });
}

});

