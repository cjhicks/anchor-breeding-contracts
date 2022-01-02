import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { BreedingCooldown } from '../target/types/breeding_cooldown';
const { SystemProgram } = anchor.web3;
const assert = require("assert");

describe('breeding_cooldown', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = (<any>anchor).workspace.BreedingCooldown as Program<BreedingCooldown>;
  const PREFIX = 'bapeBreeding';

  const user = anchor.getProvider().wallet;
  const potion = anchor.web3.Keypair.generate();

  const tokenUserAccount = anchor.web3.Keypair.generate();
  const tokenMint = anchor.web3.Keypair.generate();
  const potionMint = anchor.web3.Keypair.generate();

  const nft1 = anchor.web3.Keypair.generate();
  const nft1Metadata = anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode(PREFIX)), nft1.publicKey.toBuffer()],
    program.programId
  );

  const nft2 = anchor.web3.Keypair.generate();
  const nft2Metadata = anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode(PREFIX)), nft1.publicKey.toBuffer()],
    program.programId
  );

  // it('Creates potion with price and cooldown', async () => {
  //   await program.rpc.createPotion(wallet.publicKey, {
  //     accounts: {
  //       user: user.publicKey,
  //       potion: potion.publicKey,
  //       userTokenAccount: tokenUserAccount.publicKey,
  //       tokenMint: tokenMint.publicKey,
  //       potionMint: potionMint.publicKey,
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
