import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { BreedingCooldown } from '../target/types/breeding_cooldown';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { createMint, createTokenAccount, getTokenAccount } from '@project-serum/common';
import { mintToAccount } from './utils';
import { PublicKey } from '@solana/web3.js';

const { SystemProgram, SYSVAR_RENT_PUBKEY } = anchor.web3;
const assert = require("assert");

describe('breeding_cooldown', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const connection = anchor.getProvider().connection;
  const program = (<any>anchor).workspace.BreedingCooldown as Program<BreedingCooldown>;
  const PREFIX = 'bapeBreeding';

  const userPubKey = anchor.getProvider().wallet.publicKey;

  // async function createUserAccountWithTokens(amount: number) {
  //   // create mint, and add $BAPE to user account
  //   const mintPubKey = await createMint(program.provider, userPubKey, 0);
  //   // Create user accounts
  //   let tokenUserAccountPubKey = await createTokenAccount(program.provider, mintPubKey, userPubKey);
  //   // Fund user with 500 $BAPE
  //   await mintToAccount(program.provider, mintPubKey, tokenUserAccountPubKey, 349, userPubKey);

  //   return tokenUserAccountPubKey;
  // }

  // async function createUserNftAccount() {
  //   return await createUserAccountWithTokens(1)
  // }

  async function getNftMetadataPubKey(nft: anchor.web3.Keypair): Promise<anchor.web3.PublicKey> {
    return anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode(PREFIX)), nft.publicKey.toBuffer()],
      program.programId
    ).then(x => x[0])
  }

  it('Creates potion', async () => {
    const potion = anchor.web3.Keypair.generate();

    // create $BAPE mint, and add $BAPE to user account
    const tokenMintPubKey = await createMint(program.provider, userPubKey, 0);
    // Create user and program $BAPE accounts
    let tokenUserAccountPubKey = await createTokenAccount(program.provider, tokenMintPubKey, userPubKey);
    // Fund user with 500 $BAPE
    await mintToAccount(program.provider, tokenMintPubKey, tokenUserAccountPubKey, 500, userPubKey);
    // Create PDA's for nft metadata
    const nft1 = anchor.web3.Keypair.generate();
    const nft2 = anchor.web3.Keypair.generate();
    const nft1MetadataPubKey = await getNftMetadataPubKey(nft1);
    const nft2MetadataPubKey = await getNftMetadataPubKey(nft2);
    // Create Potion Mint
    // const potionMintPubKey = await createMint(program.provider, userPubKey, 0);
    // let potionUserAccountPubKey = anchor.web3.Keypair.generate();

    await program.rpc.createPotion(userPubKey, {
      accounts: {
        user: userPubKey,
        potion: potion.publicKey,
        tokenUserAccount: tokenUserAccountPubKey,
        tokenMint: tokenMintPubKey,
        // potionMint: potionMintPubKey,
        // potionUserAccount: potionUserAccountPubKey,
        nft1: nft1.publicKey,
        nft1Metadata: nft1MetadataPubKey,
        nft2: nft2.publicKey,
        nft2Metadata: nft2MetadataPubKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      },
      signers: [potion]
    })
    
    // Assert potion was initialized properly
    let potionAccount = await program.account.potion.fetch(potion.publicKey)
    assert(potionAccount.authority.equals(userPubKey))
    assert(potionAccount.createdTimestamp.toNumber() > 0)

    // Assert both NFT metadata match potion
    let nft1Metadata = await program.account.nftMetadata.fetch(nft1MetadataPubKey)
    assert(nft1Metadata.authority.equals(userPubKey))
    assert(nft1Metadata.lastBredTimestamp.toNumber() == potionAccount.createdTimestamp.toNumber())
    assert(nft1Metadata.nft.equals(potionAccount.nft1))

    let nft2Metadata = await program.account.nftMetadata.fetch(nft2MetadataPubKey)
    assert(nft2Metadata.authority.equals(userPubKey))
    assert(nft2Metadata.lastBredTimestamp.toNumber() == potionAccount.createdTimestamp.toNumber())
    assert(nft2Metadata.nft.equals(potionAccount.nft2))

    // Assert that 350 $BAPE was burned
    let tokenUserAccount = await getTokenAccount(program.provider, tokenUserAccountPubKey);
    assert(tokenUserAccount.amount.toNumber() == 150)
  });

  it('Fails to create 2nd potion because <7 days since nfts were used', async () => {
    const potion = anchor.web3.Keypair.generate();

    // create $BAPE mint, and add $BAPE to user account
    const tokenMintPubKey = await createMint(program.provider, userPubKey, 0);
    // Create user and program $BAPE accounts
    let tokenUserAccountPubKey = await createTokenAccount(program.provider, tokenMintPubKey, userPubKey);
    // Fund user with 1000 $BAPE (enough for 2 transactions)
    await mintToAccount(program.provider, tokenMintPubKey, tokenUserAccountPubKey, 1000, userPubKey);
    // Create PDA's for nft metadata
    const nft1 = anchor.web3.Keypair.generate();
    const nft2 = anchor.web3.Keypair.generate();
    const nft1MetadataPubKey = await getNftMetadataPubKey(nft1);
    const nft2MetadataPubKey = await getNftMetadataPubKey(nft2);

    // create once
    let promise = program.rpc.createPotion(userPubKey, {
      accounts: {
        user: userPubKey,
        potion: potion.publicKey,
        tokenUserAccount: tokenUserAccountPubKey,
        tokenMint: tokenMintPubKey,
        // potionMint: potionMintPubKey,
        nft1: nft1.publicKey,
        nft1Metadata: nft1MetadataPubKey,
        nft2: nft2.publicKey,
        nft2Metadata: nft2MetadataPubKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      },
      signers: [potion]
    })
    await promise
    
    // Create again with different potion
    let potion2 = anchor.web3.Keypair.generate();
    let promise2 = program.rpc.createPotion(userPubKey, {
      accounts: {
        user: userPubKey,
        potion: potion2.publicKey,
        tokenUserAccount: tokenUserAccountPubKey,
        tokenMint: tokenMintPubKey,
        // potionMint: potionMintPubKey,
        nft1: nft1.publicKey,
        nft1Metadata: nft1MetadataPubKey,
        nft2: nft2.publicKey,
        nft2Metadata: nft2MetadataPubKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      },
      signers: [potion2]
    })
    try {
      await promise2
    } catch (error) {
      assert((<string>error.message).endsWith('This NFT has been used for breeding in the last 7 days.'))
    }

    // Assert that 350 $BAPE was only burned once
    let tokenUserAccount = await getTokenAccount(program.provider, tokenUserAccountPubKey);
    assert(tokenUserAccount.amount.toNumber() == 650)

    // Assert 2nd potion was not created
    try {
      await program.account.potion.fetch(potion2.publicKey)
    } catch (error) {
      assert((<string>error.message).includes('Account does not exist'))
    }
  });

  it('Throws error if not enough $BAPE to create potion', async () => {
    const potion = anchor.web3.Keypair.generate();

    // create $BAPE mint, and add $BAPE to user account
    const tokenMintPubKey = await createMint(program.provider, userPubKey, 0);
    // Create user and program $BAPE accounts
    let tokenUserAccountPubKey = await createTokenAccount(program.provider, tokenMintPubKey, userPubKey);
    // Fund user with 500 $BAPE
    await mintToAccount(program.provider, tokenMintPubKey, tokenUserAccountPubKey, 349, userPubKey);
    // Create PDA's for nft metadata
    const nft1 = anchor.web3.Keypair.generate();
    const nft2 = anchor.web3.Keypair.generate();
    const nft1MetadataPubKey = await getNftMetadataPubKey(nft1);
    const nft2MetadataPubKey = await getNftMetadataPubKey(nft2);

    let promise = program.rpc.createPotion(userPubKey, {
      accounts: {
        user: userPubKey,
        potion: potion.publicKey,
        tokenUserAccount: tokenUserAccountPubKey,
        tokenMint: tokenMintPubKey,
        // potionMint: potionMintPubKey,
        nft1: nft1.publicKey,
        nft1Metadata: nft1MetadataPubKey,
        nft2: nft2.publicKey,
        nft2Metadata: nft2MetadataPubKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      },
      signers: [potion]
    })

    try {
      await promise
    } catch (error) {
      assert((<string>error.message).endsWith('User has insufficient funds to complete the transaction.'))
    }
    // Assert that $BAPE was not burned
    let tokenUserAccount = await getTokenAccount(program.provider, tokenUserAccountPubKey);
    assert(tokenUserAccount.amount.toNumber() == 349)

    // Assert potion was not created
    try {
      await program.account.potion.fetch(potion.publicKey)
    } catch (error) {
      assert((<string>error.message).includes('Account does not exist'))
    }
  });

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
