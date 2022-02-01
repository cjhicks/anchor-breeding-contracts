# anchor-breeding-contracts
Repo for our custom breeding contracts using https://github.com/project-serum/anchor

## Testing in Devnet

**Note**: To test from the frontend, you need to pull the `breeding` branch, so that you can invoke [breedPotion()](https://github.com/gabrielhicks/bape/pull/3/files#diff-789710c84584424e11991c5dadd0c97bfcaa31bd3eb1e255f75db316554211afR163-R220)

```bash
anchor build

anchor deploy
# Program Id: CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD

# Note: ProgramID needs to match line 15 of lib.rs, also needs to batch BREEDING_PROGRAM_ID in breeding.ts of front-end

# First Time: anchor idl init -f ./target/idl/breeding_cooldown.json Ajg8yy4gNuLwMWdH1k7sWVNaZb3nMu4wMHY8YED4iY6Y
anchor idl upgrade -f ./target/idl/breeding_cooldown.json 9CNNoWiwBJzQzW72ycRvZyFQLqkyiN4TkmzmNooiTBsw

# from frontend:
BROWSER=firefox npm start # or whatever browser you want to debug with

```

## Current Issues

This is the folliwing error I currently get:
```Transaction simulation failed: Error processing Instruction 4: Cross-program invocation with unauthorized signer or writable account 
    Program 11111111111111111111111111111111 invoke [1]
    Program 11111111111111111111111111111111 success
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]
    Program log: Instruction: InitializeMint
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2390 of 200000 compute units
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
    Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]
    Program log: Transfer 2039280 lamports to the associated token account
    Program 11111111111111111111111111111111 invoke [2]
    Program 11111111111111111111111111111111 success
    Program log: Allocate space for the associated token account
    Program 11111111111111111111111111111111 invoke [2]
    Program 11111111111111111111111111111111 success
    Program log: Assign the associated token account to the SPL Token program
    Program 11111111111111111111111111111111 invoke [2]
    Program 11111111111111111111111111111111 success
    Program log: Initialize the associated token account
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
    Program log: Instruction: InitializeAccount
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3449 of 171989 compute units
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
    Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 32109 of 200000 compute units
    Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]
    Program log: Instruction: MintTo
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2879 of 200000 compute units
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
    Program CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD invoke [1]
    Program log: Instruction: CreatePotion
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
    Program log: Instruction: Burn
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2766 of 178904 compute units
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
    Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s invoke [2]
    Program log: (Deprecated as of 1.1.0) Instruction: Create Metadata Accounts
    Program log: Transfer 5616720 lamports to the new account
    FCoZfkAQm1sqCj9s6LtBus2EDZqYQ4THRPkoM91Uhgeq's writable privilege escalated
    Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s consumed 9245 of 169177 compute units
    Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s failed: Cross-program invocation with unauthorized signer or writable account
    Program CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD consumed 40068 of 200000 compute units
    Program CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD failed: Cross-program invocation with unauthorized signer or writable account index.js:1
```

## Suggestions
I think something is wrong with the "Creators" that go into the `create_metadata_accounts()` instruction on lines 128-172.


I get errors when I use these accounts, I've tried creating them myself too and still gotten the same error. 

If you take `potion_creator` and `other_creator` out of the contract (and redeploy the program+IDL), you can create the token from the frontend. Example token creation: https://explorer.solana.com/tx/3Wgr2FztfGwvKzhnEATSPxa9KUK7jo1vy4RLazw1eRQpfC99ems1TNbHhBUojYVwbBKSEP2mEzgi1aV4F2eJpobD?cluster=devnet

The pattern I'm using looks nearly identical to the following examples, but I still can't seem to get it right:

- [Dragonz frontend](https://github.com/gabrielhicks/bapeBreeding/blob/master/src/contracts/breeding.ts#L205-L271) and [Breeding Instruction](https://explorer.solana.com/tx/g5fg51XveddE1MyU3GsEUpU6e3vUz1BhWNBvye6hBziDZbKsBv4H1UjLEKr1rjLFtABt6YNM6TBBoMzDxtQ5td5)
- Token Breeding 2 example [frontend](https://github.com/gabrielhicks/tokenBreeding2/blob/main/potion-breeding-client-v2/src/main.rs#L324-L356) and [backend](https://github.com/gabrielhicks/tokenBreeding2/blob/main/potion-breeding-contract-v2/src/lib.rs#L620-L647)
- Candy Machine [backend](https://github.com/metaplex-foundation/metaplex-program-library/blob/master/nft-candy-machine/program/src/lib.rs#L316-L334)
- [Metaboss](https://github.com/samuelvanderwaal/metaboss/blob/edeb9acdb63dc53278c66ffec4d0509b8304c5b7/src/mint.rs#L283-L297)

If I just use my user wallet, sometimes I can get to the point where I get a different error: "instruction is missing account", but we need to use the same wallet as Creator 0, so I don't think this is the right way to go.

If I change `invoke_signed()` to `invoke()`, I can get an "unauthorized writeable account in instruction", but I think this is because I'm not signing the instruction in that case.

### Next Steps
If we get `create_metadata_accounts()` working, the next two instructions (`create_master_edition()` and `update_metadata_accounts()`) are stubbed and commmented out. 
They follow a very similar pattern, so I'm hopeful these will be straightforward once we get `create_metadata_accounts()` working!

## Troubleshooting

If you create a token and get the "bred in less than 10 days" error, you can change `PREFIX` to something else (currently bapeBreedingTest16). You also need to change `PREFIX` in breeding.ts to match (line 39).
