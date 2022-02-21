# anchor-breeding-contracts
Repo for our custom breeding contracts using https://github.com/project-serum/anchor

## Testing in Devnet

**Note**: To test from the frontend, you need to pull the `breeding` branch, so that you can invoke [breedPotion()](https://github.com/gabrielhicks/bape/pull/3/files#diff-789710c84584424e11991c5dadd0c97bfcaa31bd3eb1e255f75db316554211afR163-R220)

```bash
anchor build

anchor deploy
# Program Id: CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD

# Note: ProgramID needs to match line 15 of lib.rs, also needs to batch BREEDING_PROGRAM_ID in breeding.ts of front-end

# First Time: 
anchor idl init -f ./target/idl/breeding_cooldown.json Ajg8yy4gNuLwMWdH1k7sWVNaZb3nMu4wMHY8YED4iY6Y
# On update:
anchor idl upgrade -f ./target/idl/breeding_cooldown.json 9CNNoWiwBJzQzW72ycRvZyFQLqkyiN4TkmzmNooiTBsw

# from frontend:
BROWSER=firefox npm start # or whatever browser you want to debug with

```

## Troubleshooting

If you create a token and get the "bred in less than 10 days" error, you can change `PREFIX` to something else (currently bapeBreedingTest16). You also need to change `PREFIX` in breeding.ts to match (line 39).
