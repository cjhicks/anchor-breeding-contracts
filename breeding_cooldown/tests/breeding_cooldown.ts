import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { BreedingCooldown } from '../target/types/breeding_cooldown';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { createMint, createTokenAccount, getTokenAccount } from '@project-serum/common';
import { mintToAccount } from './utils';

const { SystemProgram, SYSVAR_RENT_PUBKEY } = anchor.web3;
const assert = require("assert");

describe('breeding_cooldown', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const connection = anchor.getProvider().connection;
  const program = (<any>anchor).workspace.BreedingCooldown as Program<BreedingCooldown>;
  const PREFIX = 'bapeBreeding';

  const userPubKey = anchor.getProvider().wallet.publicKey;
  const potionMintPubKey = anchor.web3.Keypair.generate().publicKey; // new anchor.web3.PublicKey("29oqZtZxzytxuSPHVB3GaFRXR9GtEZbsdp7rFd4JsTrM"); // 

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

    await program.rpc.createPotion(userPubKey, {
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
    
    // Assert potion was initialized properly
    let potionAccount = await program.account.potion.fetch(potion.publicKey)
    assert(potionAccount.authority.equals(userPubKey))
    assert(potionAccount.createdTimestamp.toNumber() > 0)

    // Assert both NFT metadata match potion
    let nft1Metadata = await program.account.nftMetadata.fetch(nft1MetadataPubKey)
    assert(nft1Metadata.authority.equals(userPubKey))
    assert(nft1Metadata.lastBredTimestamp.toNumber() == potionAccount.createdTimestamp.toNumber())

    let nft2Metadata = await program.account.nftMetadata.fetch(nft2MetadataPubKey)
    assert(nft2Metadata.authority.equals(userPubKey))
    assert(nft2Metadata.lastBredTimestamp.toNumber() == potionAccount.createdTimestamp.toNumber())

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
    // Fund user with 500 $BAPE
    await mintToAccount(program.provider, tokenMintPubKey, tokenUserAccountPubKey, 500, userPubKey);
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
    
    try {
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
      await promise2
    } catch (error) {
      assert((<string>error.message) == '6000: This NFT has been used for breeding in the last 7 days.')
    }
  });

  // TODO: test insufficient $BAPE

  // TODO: succeeds if NFT's breeded more than 7 days ago

  // TODO: test metadata prefix is enforced

  // it('Creates potion with price and cooldown', async () => {
  //   await program.rpc.createPotion(wallet.publicKey, {
  //     accounts: {
  //       potion: potion.publicKey,
  //       user: wallet.publicKey,
  //       systemProgram: SystemProgram.programId
  //     },
  //     signers: [potion]
  //   })
    
  //   let potionAccount = await program.account.potion.fetch(potion.publicKey)

  //   assert(potionAccount.authority.equals(wallet.publicKey))
  //   assert(potionAccount.createdTimestamp.toNumber() > 0)
  //   assert(potionAccount.price.toNumber() == 5)
  //   assert(potionAccount.cooldownDays.toNumber() == 7)
  // });

  // it('Throws error if potion is created twice', async () => {
  //   let promise = program.rpc.createPotion(wallet.publicKey, {
  //     accounts: {
  //       potion: potion.publicKey,
  //       user: wallet.publicKey,
  //       systemProgram: SystemProgram.programId
  //     },
  //     signers: [potion]
  //   })

  //   try {
  //     await promise
  //   } catch (error) {
  //     console.log(<string>error.message)
  //     assert((<string>error.message) == 'failed to send transaction: Transaction simulation failed: This transaction has already been processed')
  //   }
  // });

  // it('Throws error if potion cooldown not reached', async () => {
  //   let promise = program.rpc.react({
  //     accounts: {
  //       potion: potion.publicKey,
  //       authority: wallet.publicKey
  //     }
  //   })

  //   try {
  //     await promise
  //   } catch (error) {
  //     assert((<string>error.message) == '6001: This potion has not reached its cooldown period.')
  //   }
  // });

  // it('Increments counter', async () => {
  //   await program.rpc.increment({
  //     accounts: {
  //       counter: counter.publicKey,
  //       authority: wallet.publicKey
  //     }
  //   })

  //   let counterAccount = await program.account.counter.fetch(counter.publicKey)

  //   assert(counterAccount.authority.equals(wallet.publicKey))
  //   assert(counterAccount.count.toNumber() == 1)
  // });

  // it('Throws error if wallet is not authorized to increment', async () => {
  //   let wrongWalletPubKey = anchor.web3.Keypair.generate();

  //   let promise = program.rpc.increment({
  //     accounts: {
  //       counter: counter.publicKey,
  //       authority: wrongWalletPubKey
  //     }
  //   })

  //   try {
  //     await promise
  //   } catch (error) {
  //     assert((<string>error.message) == 'Wrong input type for account "authority" in the instruction accounts object for instruction "increment". Expected PublicKey or string.')
  //   }

  //   // assert counter left unchanged
  //   let counterAccount = await program.account.counter.fetch(counter.publicKey)
  //   assert(counterAccount.authority.equals(wallet.publicKey))
  //   assert(counterAccount.count.toNumber() == 1)
  // });

});
