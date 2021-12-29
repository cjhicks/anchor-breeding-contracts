import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { BreedingCooldown } from '../target/types/breeding_cooldown';
const { SystemProgram } = anchor.web3;
const assert = require("assert");

describe('breeding_cooldown', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const wallet = anchor.getProvider().wallet;

  const counter = anchor.web3.Keypair.generate();

  const program = (<any>anchor).workspace.BreedingCooldown as Program<BreedingCooldown>;

  it('Initializes counter to 0', async () => {
    await program.rpc.initialize(wallet.publicKey, {
      accounts: {
        counter: counter.publicKey,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId
      },
      signers: [counter]
    })
    
    let counterAccount = await program.account.counter.fetch(counter.publicKey)

    assert(counterAccount.authority.equals(wallet.publicKey))
    assert(counterAccount.count.toNumber() == 0)
  });

  it('Increments counter', async () => {
    await program.rpc.increment({
      accounts: {
        counter: counter.publicKey,
        authority: wallet.publicKey
      }
    })

    let counterAccount = await program.account.counter.fetch(counter.publicKey)

    assert(counterAccount.authority.equals(wallet.publicKey))
    assert(counterAccount.count.toNumber() == 1)
  });

  it('Throws error if wallet is not authorized to increment', async () => {
    let wrongWalletPubKey = anchor.web3.Keypair.generate();

    let promise = program.rpc.increment({
      accounts: {
        counter: counter.publicKey,
        authority: wrongWalletPubKey
      }
    })

    try {
      await promise
    } catch (error) {
      assert((<string>error.message) == 'Wrong input type for account "authority" in the instruction accounts object for instruction "increment". Expected PublicKey or string.')
    }

    // assert counter left unchanged
    let counterAccount = await program.account.counter.fetch(counter.publicKey)
    assert(counterAccount.authority.equals(wallet.publicKey))
    assert(counterAccount.count.toNumber() == 1)
  });

});
