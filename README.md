# anchor-breeding-contracts
Repo for our custom breeding contracts using https://github.com/project-serum/anchor

## Testing in Devnet
```bash
anchor build

anchor deploy
# Program Id: CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD

# Note: ProgramID needs to match line 15 of lib.rs, also needs to batch BREEDING_PROGRAM_ID in breeding.ts of front-end

anchor idl upgrade -f ./target/idl/breeding_cooldown.json CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD
# use same program ID from above here to deploy the IDL

# from frontend:
BROWSER=firefox npm start # or whatever browser you want to debug with

```

## Current Issues

This is the folliwing error I currently get:
```
Transaction simulation failed: Error processing Instruction 4: Program failed to complete 
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
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3449 of 177047 compute units
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
    Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 27051 of 200000 compute units
    Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]

    Program log: Instruction: MintTo
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2879 of 200000 compute units
    Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success

    Program CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD invoke [1]
    Program CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD consumed 3609 of 200000 compute units
    Program failed to complete: Access violation in stack frame 3 at address 0x200003f40 of size 8 by instruction #9931
    Program CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD failed: Program failed to complete index.js:1
​```