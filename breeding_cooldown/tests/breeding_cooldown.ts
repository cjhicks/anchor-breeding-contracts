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

  // TODO: pull into utils
  async function pda(seeds: Buffer[]): Promise<[anchor.web3.PublicKey, number]> {
    return anchor.web3.PublicKey.findProgramAddress(seeds, program.programId)
  }

  async function nftStateAddress(nftKey: anchor.web3.PublicKey): Promise<[anchor.web3.PublicKey, number]> {
    return pda([PREFIX, nftKey.toBuffer()]);
  }

  async function potionCountAddress(): Promise<[anchor.web3.PublicKey, number]> {
    return pda([PREFIX, PREFIX_COUNT]);
  }

  async function potionStateAddress(potionMintKey: anchor.web3.PublicKey): Promise<[anchor.web3.PublicKey, number]> {
    return pda([PREFIX, potionMintKey.toBuffer()]);
  }

  async function potionCreatorAddress(): Promise<[anchor.web3.PublicKey, number]> {
    return pda([PREFIX, PREFIX_POTION]);
  }

  async function metadataAddress(mint: PublicKey): Promise<[anchor.web3.PublicKey, number]> {
    return anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )
  }

  async function masterEditionAddress(mint: PublicKey): Promise<[anchor.web3.PublicKey, number]> {
    return anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
        Buffer.from('edition'),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )
  }

  async function associatedTokenAddress(mint: PublicKey): Promise<[anchor.web3.PublicKey, number]> {
    return anchor.web3.PublicKey.findProgramAddress(
      [
        userPubKey.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )
  }

  async function preInstructions(mintKey: PublicKey, tokenKey: PublicKey): Promise<anchor.web3.TransactionInstruction[]> {
    const rent = await connection.getMinimumBalanceForRentExemption(
      MintLayout.span
    );
    return [
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: userPubKey,
        newAccountPubkey: mintKey,
        space: MintLayout.span,
        lamports: rent,
        programId: TOKEN_PROGRAM_ID,
      }),
      Token.createInitMintInstruction(
        TOKEN_PROGRAM_ID,
        mintKey,
        0,
        userPubKey,
        userPubKey
      ),
      createAssociatedTokenAccountInstruction(
        tokenKey,
        userPubKey,
        userPubKey,
        mintKey
      ),
      Token.createMintToInstruction(
        TOKEN_PROGRAM_ID,
        mintKey,
        tokenKey,
        userPubKey,
        [],
        1
      ),
    ]
  }

  it('Initializes a vector of URIs',async () => {
      const [urisKey, _] = await pda([PREFIX, PREFIX_URI])

      await program.rpc.initUris({
        accounts: {
          user: userPubKey,
          uris: urisKey,
          systemProgram: SystemProgram.programId
        }
      })
    
      // Assert URI is an empty vector
      let uris = await program.account.uris.fetch(urisKey)
      console.log(uris);
      // assert((uris.relativeUris as string[]).length == 0);
  })

  it('Adds URIs to vector',async () => {
    const [urisKey, _] = await pda([PREFIX, PREFIX_URI])

    // with 10k bytes, got to 248 strings. 50k bytes is too big
    // using Vec<u8> got same thing... can we compress bytes to an int?
    let NUM_URIS = 1000; 
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

    let uris = await program.account.uris.fetch(urisKey)
    console.log(uris)

    const deserialized = anchor.web3.Keypair.generate();
    await program.rpc.deserializeUri(0, {
      accounts: {
        user: userPubKey,
        uris: urisKey,
        deserialized: deserialized.publicKey,
        systemProgram: SystemProgram.programId
      },
      signers: [deserialized]
    })

    let deserializedUri = await program.account.deserializedUri.fetch(deserialized.publicKey);
    console.log(deserializedUri.relativeUri);
    // let relativeUris = uris.relativeUris as string[]
    // assert all URI's are there
    // assert(relativeUris.length == NUM_URIS);
    // Assert URI's are returned in order they're added
    // for (let i = 0; i < NUM_URIS; i++) {
    //   assert(relativeUris[i] == expectedUris[i]);
    // }
})

  // it('Creates first potion', async () => {
  //   // Potion
  //   const [potionCountKey, ] = await potionCountAddress();
  //   const potionMint = anchor.web3.Keypair.generate();
  //   // const potionMintKey = await createMint(program.provider, userPubKey, 0);
  //   // const [potionMint, ] = await pda([potionMintKey.toBuffer()]);
  //   const [potionStateKey, ] = await potionStateAddress(potionMint.publicKey);
  //   const [potionCreatorKey, potionCreatorBump] = await potionCreatorAddress();
  //   const [potionMintMetadataKey, ] = await metadataAddress(potionMint.publicKey);
  //   const [potionMasterEditionKey, ] = await masterEditionAddress(potionMint.publicKey);
  //   const [potionTokenKey, ] = await associatedTokenAddress(potionMint.publicKey);

  //   // BAPE
  //   const tokenMintKey = await createMint(program.provider, userPubKey, 0);
  //   let tokenUserAccountKey = await createTokenAccount(program.provider, tokenMintKey, userPubKey);
  //   await mintToAccount(program.provider, tokenMintKey, tokenUserAccountKey, 500, userPubKey);

  //   // Parents
  //   const nft1 = anchor.web3.Keypair.generate();
  //   const nft2 = anchor.web3.Keypair.generate();
  //   const [nft1StateKey, ] = await nftStateAddress(nft1.publicKey);
  //   const [nft2StateKey, ] = await nftStateAddress(nft2.publicKey);

  //   // const instructions = await preInstructions(potionMint.publicKey, potionTokenKey);
  
  //   console.log(potionMint.publicKey.toString());

  //   await program.rpc.createPotion(potionCreatorBump, {
  //     accounts: {
  //       user: userPubKey,
  //       potionCount: potionCountKey,
  //       potionMint: potionMint.publicKey,
  //       potionState: potionStateKey,
  //       potionCreator: potionCreatorKey,
  //       potionMintMetadata: potionMintMetadataKey,
  //       potionMasterEdition: potionMasterEditionKey,
  //       potionToken: potionTokenKey,
  //       tokenUserAccount: tokenUserAccountKey,
  //       tokenMint: tokenMintKey,
  //       nft1: nft1.publicKey,
  //       nft1State: nft1StateKey,
  //       nft2: nft2.publicKey,
  //       nft2State: nft2StateKey,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //       rent: SYSVAR_RENT_PUBKEY
  //     },
  //     signers: [potionMint],
  //     // preInstructions: instructions
  //   })
    
  //   // Assert potion was initialized properly
  //   let potionState = await program.account.potionState.fetch(potionStateKey)
  //   assert(potionState.createdTimestamp.toNumber() > 0)

  //   // TODO: potion count

  //   // // Assert both NFT metadata match potion
  //   // let nft1Metadata = await program.account.nftMetadata.fetch(nft1MetadataPubKey)
  //   // assert(nft1Metadata.authority.equals(userPubKey))
  //   // assert(nft1Metadata.lastBredTimestamp.toNumber() == potionAccount.createdTimestamp.toNumber())
  //   // assert(nft1Metadata.nft.equals(potionAccount.nft1))

  //   // let nft2Metadata = await program.account.nftMetadata.fetch(nft2MetadataPubKey)
  //   // assert(nft2Metadata.authority.equals(userPubKey))
  //   // assert(nft2Metadata.lastBredTimestamp.toNumber() == potionAccount.createdTimestamp.toNumber())
  //   // assert(nft2Metadata.nft.equals(potionAccount.nft2))

  //   // // Assert that 350 $BAPE was burned
  //   // let tokenUserAccount = await getTokenAccount(program.provider, tokenUserAccountPubKey);
  //   // assert(tokenUserAccount.amount.toNumber() == 150)
  // });

  // it('Fails to create 2nd potion because <7 days since nfts were used', async () => {
  //   const potion = anchor.web3.Keypair.generate();

  //   // create $BAPE mint, and add $BAPE to user account
  //   const tokenMintPubKey = await createMint(program.provider, userPubKey, 0);
  //   // Create user and program $BAPE accounts
  //   let tokenUserAccountPubKey = await createTokenAccount(program.provider, tokenMintPubKey, userPubKey);
  //   // Fund user with 1000 $BAPE (enough for 2 transactions)
  //   await mintToAccount(program.provider, tokenMintPubKey, tokenUserAccountPubKey, 1000, userPubKey);
  //   // Create PDA's for nft metadata
  //   const nft1 = anchor.web3.Keypair.generate();
  //   const nft2 = anchor.web3.Keypair.generate();
  //   const nft1MetadataPubKey = await getNftMetadataPubKey(nft1);
  //   const nft2MetadataPubKey = await getNftMetadataPubKey(nft2);

  //   // create once
  //   let promise = program.rpc.createPotion(userPubKey, {
  //     accounts: {
  //       user: userPubKey,
  //       potion: potion.publicKey,
  //       tokenUserAccount: tokenUserAccountPubKey,
  //       tokenMint: tokenMintPubKey,
  //       // potionMint: potionMintPubKey,
  //       nft1: nft1.publicKey,
  //       nft1Metadata: nft1MetadataPubKey,
  //       nft2: nft2.publicKey,
  //       nft2Metadata: nft2MetadataPubKey,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //       rent: SYSVAR_RENT_PUBKEY
  //     },
  //     signers: [potion]
  //   })
  //   await promise
    
  //   // Create again with different potion
  //   let potion2 = anchor.web3.Keypair.generate();
  //   let promise2 = program.rpc.createPotion(userPubKey, {
  //     accounts: {
  //       user: userPubKey,
  //       potion: potion2.publicKey,
  //       tokenUserAccount: tokenUserAccountPubKey,
  //       tokenMint: tokenMintPubKey,
  //       // potionMint: potionMintPubKey,
  //       nft1: nft1.publicKey,
  //       nft1Metadata: nft1MetadataPubKey,
  //       nft2: nft2.publicKey,
  //       nft2Metadata: nft2MetadataPubKey,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //       rent: SYSVAR_RENT_PUBKEY
  //     },
  //     signers: [potion2]
  //   })
  //   try {
  //     await promise2
  //   } catch (error) {
  //     assert((<string>error.message).endsWith('This NFT has been used for breeding in the last 7 days.'))
  //   }

  //   // Assert that 350 $BAPE was only burned once
  //   let tokenUserAccount = await getTokenAccount(program.provider, tokenUserAccountPubKey);
  //   assert(tokenUserAccount.amount.toNumber() == 650)

  //   // Assert 2nd potion was not created
  //   try {
  //     await program.account.potion.fetch(potion2.publicKey)
  //   } catch (error) {
  //     assert((<string>error.message).includes('Account does not exist'))
  //   }
  // });

  // TODO: verify Owner actually own's these NFT's

  // TODO: Regular Reaction tests
  // TODO: verify regular reaction is authorized
  // TODO: verify NFT's match potion
  // TODO: verify 7 day window has passed


  // TODO: Fast Reaction tests
  // TODO: verify regular reaction is authorized
  // TODO: verify NFT's match potion
  // TODO: verify user has enough $BAPE to burn

});

