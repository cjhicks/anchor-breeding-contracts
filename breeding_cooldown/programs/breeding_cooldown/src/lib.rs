use anchor_lang::prelude::*;
use anchor_spl;
use solana_program::{
    sysvar::{clock::Clock, self},
    system_instruction
};
use solana_program::program::{invoke_signed, invoke};
#[allow(unused_imports)]
use spl_token_metadata::{
        instruction::{update_metadata_accounts, create_metadata_accounts, CreateMasterEditionArgs, MetadataInstruction},
        state::{Creator, Data, Metadata, MAX_URI_LENGTH, MAX_NAME_LENGTH},
};
use solana_program::instruction::{Instruction,AccountMeta};

declare_id!("CT6NTh1hRHykX69Qm5oAovPPrxeJV43hqmUA2MhmaorD");

fn get_timestamp() -> u64 {
    return Clock::get().unwrap().unix_timestamp as u64;
}

fn get_breed_min_timestamp(timestamp: u64) -> u64 {
    let seven_days_in_seconds = 7 * 24 * 60 * 60;
    return timestamp - seven_days_in_seconds;
}

const PREFIX: &str = "bapeBreedingTest16";
const PREFIX_POTION: &str = "potion";

#[program]
pub mod breeding_cooldown {
    use super::*;

    /*
    This function is equivalent to breeding an egg: https://explorer.solana.com/tx/g5fg51XveddE1MyU3GsEUpU6e3vUz1BhWNBvye6hBziDZbKsBv4H1UjLEKr1rjLFtABt6YNM6TBBoMzDxtQ5td5
    */
    pub fn create_potion(ctx: Context<CreatePotion>) -> ProgramResult {
        // let CANDY_MACHINE_ID = "6vHjQxYUwk9DNuJNHfRWSfH1UTuikVayP9h3H4iYW2TD";

        let potion_mint = &mut ctx.accounts.potion_mint;
        let potion_state = &mut ctx.accounts.potion_state;
        let potion_mint_metadata = &mut ctx.accounts.potion_mint_metadata;
        let potion_master_edition = &ctx.accounts.potion_master_edition;
        let potion_token = &ctx.accounts.potion_token;
        let potion_creator = &ctx.accounts.potion_creator;
        let other_creator = &ctx.accounts.other_creator;

        let user = &ctx.accounts.user;
        let token_mint = ctx.accounts.token_mint.to_account_info();

        let token_program = &ctx.accounts.token_program;
        let token_metadata_program = &ctx.accounts.token_metadata_program;
        let system_program = &ctx.accounts.system_program.to_account_info();
        let rent = &ctx.accounts.rent;

        /*
        Validation
        */
        // check token is $bape - change for prod
        let bape_mint = "2RTsdGVkWJU7DG77ayYTCvZctUVz3L9Crp9vkMDdRt4Y".parse::<Pubkey>().unwrap();
        if token_mint.key() != bape_mint {
            return Err(ErrorCode::WrongToken.into())
        }

        // check NFT's are not same
        let nft_1 = &ctx.accounts.nft_1;
        let nft_2 = &ctx.accounts.nft_2;
        if nft_1.key() == nft_2.key() {
            return Err(ErrorCode::SameNFTs.into())
        }

        // Check update authority - change for prod
        // let update_authority = "2RTsdGVkWJU7DG77ayYTCvZctUVz3L9Crp9vkMDdRt4Y".parse::<Pubkey>().unwrap();
        // nft_1.data().update_authority

        // // check if we have enough $BAPE before continuing
        let token_user_account = &ctx.accounts.token_user_account;
        let decimals = 9;
        let base: u64 = 10;
        let burn_price = 500 * base.pow(decimals);
        if token_user_account.amount < burn_price {
            return Err(ErrorCode::InsufficientFunds.into())
        }

        // check if 7 days since last breeding
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);
        let nft_1_state = &mut ctx.accounts.nft_1_state;
        if nft_1_state.last_bred_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }
        let nft_2_state = &mut ctx.accounts.nft_2_state;
        if nft_2_state.last_bred_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }

        // set metadata
        let nft_1_key = *ctx.accounts.nft_1.key;
        let nft_2_key = *ctx.accounts.nft_2.key;
        potion_state.authority = *user.key;
        potion_state.nft1 = nft_1_key;
        potion_state.nft2 = nft_2_key;
        potion_state.created_timestamp = timestamp;

        nft_1_state.authority = *user.key;
        nft_1_state.nft = nft_1_key;
        nft_1_state.last_bred_timestamp = timestamp;
        nft_2_state.authority = *user.key;
        nft_2_state.nft = nft_2_key;
        nft_2_state.last_bred_timestamp = timestamp;

        /*
        Burn $BAPE after minting potion
        */
        let burn_ctx = CpiContext::new(
            token_program.clone(),
            anchor_spl::token::Burn {
                to: token_user_account.to_account_info(),
                mint: token_mint,
                authority: user.to_account_info(),
            }
        );
        anchor_spl::token::burn(burn_ctx, burn_price)
            .expect("burn failed.");

        /* 
        Mint new NFT for potion
        */
        let uri = r"https://bafybeibhsnl5sz32jdfdwfvj4qea3at25wobxenzwjdirrdr2h3i4u4y2a.ipfs.infura-ipfs.io";
        let mut creators_ptn = vec![
            Creator{
                address: potion_creator.key(),
                verified: true,
                share: 0,
            },
            Creator{
                address: other_creator.key(),
                verified: false,
                share: 100,
            },
        ];
        let create_metadata_ix = &create_metadata_accounts(    
            token_metadata_program.key(),// spl_token_metadata::id(), 
            potion_mint_metadata.key(),
            potion_mint.key(),
            user.key(),
            user.key(),
            user.key(),
            "Potion".to_string(),
            "PTN".to_string(),
            uri.to_string(),
            Some(creators_ptn),
            0, //royalties,
            true,
            true, // false?
        );

        // fine until here
        invoke_signed( // 
            create_metadata_ix,
            &[
                potion_mint_metadata.clone(),
                potion_mint.to_account_info(),
                user.to_account_info(),
                potion_creator.to_account_info(),
                other_creator.to_account_info(),
                token_program.to_account_info(),
                token_metadata_program.to_account_info(),
                system_program.clone(),
                rent.to_account_info()
            ],
            &[&[PREFIX.as_bytes(), PREFIX_POTION.as_bytes()]],
        ).expect("create_metadata_accounts failed.");

        // invoke_signed(
        //     &create_master_edition(
        //         token_metadata_program.key(), 
        //         potion_master_edition.key(),
        //         potion_mint.key(),
        //         user.key(),
        //         user.key(),
        //         potion_mint_metadata.key(),
        //         user.key(),
        //         Some(0),
        //     ),
        //     &[  
        //         potion_master_edition.clone().to_account_info(),
        //         potion_mint.clone().to_account_info(),
        //         // price_data_info.clone().to_account_info(),
        //         user.clone().to_account_info(),
        //         potion_mint_metadata.clone().to_account_info(),
        //         potion_token.clone().to_account_info(),
        //         token_metadata_program.to_account_info()
        //     ],
        //     &[&[&"price".as_bytes(), &[price_bump]]],
        // )?;

        // invoke_signed(
        //     &update_metadata_accounts(
        //         spl_token_metadata::id(), 
        //         potion_mint_metadata.key(),
        //         price_data_info_key,
        //         None,
        //         None,
        //         Some(true),
        //     ),
        //     &[  
        //         potion_mint_metadata.clone().to_account_info(),
        //         // price_data_info.clone(),
        //         token_metadata_program.to_account_info()
        //     ],
        //     &[&[&"price".as_bytes(), &[price_bump]]],
        // )?;

        Ok(())
    }

    pub fn react(ctx: Context<React>) -> ProgramResult {

        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);

        /*
        Validations (function)
        1. User is Authority on Potion
        2. NFT metadata matches Potion and Authority
        3. Created Timestamp > 7 days
        4. Verify Mint on egg is legit
        */
        let user_key = *ctx.accounts.user.key;
        let potion = &mut ctx.accounts.potion;
        // let nft_1_state = &ctx.accounts.nft_1_state;
        // let nft_2_state = &ctx.accounts.nft_2_state;

        // TODO: Token Owner is me
        // TODO: Harcoded Potion Mint ID?

        // TODO: mint before hand? init mint?
        // TODO: Candy machine with same image
        // Update authority - one created (wallet)

        // mint authority - one that minted it - can't create?
        // on egg, mint authority is metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s - Metaplex Token Metadata
        // https://solscan.io/token/DjcECAj4TYJANgr9oFmcZTcdJrA5icnoscs5k3CddbVS

        // TODO: createAssociatedTokenAccountInstruction(potionToken, walletKey, walletKey, potionMint),
        // TODO: Transfer Potion

        if potion.authority != user_key { //|| nft_1_state.authority != user_key || nft_2_state.authority != user_key {
            return Err(ErrorCode::Unauthorized.into());
        }

        if potion.created_timestamp > breed_min_timestamp {
            return Err(ErrorCode::CooldownNotReached.into());
        }

        // TODO: mint new NFT (master edition)
        // TODO: make this a reusable function
        // TODO: for now, maybe follow this? Then hook into candy machine later: https://spl.solana.com/token#example-create-a-non-fungible-token

        // TODO: Create baby
        // client: use PublicKey.findProgramAddress to create empty (existing) address (state) for each NFT input
        // server: verify egg over 7 days - no parent necessary?
        // - or, should we require pass it in to verify its still held?

        // FIrst Instructions
        // 1 - Create Account (walletKey, new=babyMint)
        // 2 - Init Mint (walletKey, mint: babyMint)
        // 3 - Associated Token Create (account=babyToken, mint=babyMint, walletKey)
        // 4 - MintTo (token=babyMint, account=babyToken, mint=BabyMint, authority=walletKey)

        // Unknown Program Instructions

        Ok(())
    }

    pub fn fast_react(ctx: Context<FastReact>) -> ProgramResult {
        /*
        Validations (function)
        1. User is Authority on Potion
        2. NFT metadata matches Potion and Authority
        3. User has enough $BAPE
        4. Verify Mint on egg is legit
        */
        let user_key = *ctx.accounts.user.key;
        let potion = &mut ctx.accounts.potion;
        let nft_1_state = &ctx.accounts.nft_1_state;
        let nft_2_state = &ctx.accounts.nft_2_state;
        let token_program = &ctx.accounts.token_program;
        let user = &ctx.accounts.user;
        let token_mint = ctx.accounts.token_mint.to_account_info();

        if potion.authority != user_key || nft_1_state.authority != user_key || nft_2_state.authority != user_key {
            return Err(ErrorCode::Unauthorized.into());
        }

        // if !((potion.nft1 == nft_1_state.nft && potion.nft2 == nft_2_state.nft) ||
        //     (potion.nft1 == nft_2_state.nft && potion.nft2 == nft_1_state.nft)) {
        //     return Err(ErrorCode::Mismatch.into());
        // }
        // TODO: do fast reaction (burn more $BAPE?)
        let token_user_account = &ctx.accounts.token_user_account;
        let fast_burn_price = 250;
        if token_user_account.amount < fast_burn_price {
            return Err(ErrorCode::InsufficientFunds.into())
        }

        // TODO: mint new NFT
        // anchor_spl::token::transfer(ctx: CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>, amount: u64)

        // TODO: after mint successful, burn 175 $BAPE
        let burn_ctx = CpiContext::new(
            token_program.clone(),
            anchor_spl::token::Burn {
                to: token_user_account.to_account_info(),
                mint: token_mint,
                authority: user.to_account_info(),
            }
        );
        anchor_spl::token::burn(burn_ctx, fast_burn_price)
            .expect("burn failed.");

        Ok(())
    }
}

#[account]
pub struct CreatorData {
    // pub created_timestamp: u64
}

#[account]
#[derive(Default)]
pub struct PotionState {
    pub authority: Pubkey,
    pub nft1: Pubkey,
    pub nft2: Pubkey,
    pub created_timestamp: u64
}

#[account]
#[derive(Default)]
pub struct NftState {
    pub authority: Pubkey,
    pub nft: Pubkey,
    pub last_bred_timestamp: u64
}

#[account]
pub struct NftMetadata {
    // pub key: Key,
    pub update_authority: Pubkey,
    // pub mint: Pubkey,
    // pub data: Data,
    // // Immutable, once flipped, all sales of this metadata are considered secondary.
    // pub primary_sale_happened: bool,
    // // Whether or not the data struct is mutable, default is not
    // pub is_mutable: bool,
}

#[derive(Accounts)]
pub struct CreatePotion<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub potion_mint: Signer<'info>,
    #[account(init, payer = user, space = 8 + 120, seeds = [PREFIX.as_ref(), potion_mint.key.as_ref()], bump)]
    pub potion_state: Account<'info, PotionState>,
    // #[account(init_if_needed, payer = user, space = 8 + 8, seeds = [PREFIX.as_ref(), PREFIX_POTION.as_ref()], bump)]
    #[account(mut)]
    pub potion_creator: AccountInfo<'info>,
    #[account(mut)]
    pub other_creator: AccountInfo<'info>,
    #[account(mut)]
    pub potion_mint_metadata: AccountInfo<'info>,
    #[account(mut)]
    pub potion_master_edition: AccountInfo<'info>,
    #[account(mut)]
    pub potion_token: AccountInfo<'info>,

    // TODO: owner = user?
    #[account(mut)]
    pub token_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,  // $BAPE mint, generic enough for any token though
    // #[account(owner = *user.key)]
    pub nft_1: AccountInfo<'info>,
    // TODO: come back for validations
    // constraint= config.to_account_info().owner
    // #[account(owner = token_metadata_program, seeds = [b"metadata".as_ref(), token_metadata_program.key.as_ref(), nft_1.key.as_ref()], bump)]
    // pub nft_1_metadata: Account<'info, NftMetadata>,
    #[account(init_if_needed, seeds = [PREFIX.as_bytes(), nft_1.key.as_ref()], bump, payer = user, space = 8 + 80)]
    pub nft_1_state: Account<'info, NftState>,
    // #[account(owner = *user.key)]
    pub nft_2: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [PREFIX.as_bytes(), nft_2.key.as_ref()], bump, payer = user, space = 8 + 80)]
    pub nft_2_state: Account<'info, NftState>,

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    // #[account(address = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s".as_ref())]
    pub token_metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>, // this is just anchor.web3.SystemProgram.programId from frontend
    // pub rent: AccountInfo<'info>, // this just anchor.web3.SYSVAR_RENT_PUBKEY from frontend
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct React<'info> {
    pub user: Signer<'info>,
    #[account(mut, owner = *user.key)]
    pub potion: Account<'info, PotionState>,

    // don't need this
    // pub nft_1: AccountInfo<'info>,
    // #[account(seeds = [b"bapeBreeding".as_ref(), nft_1.key.as_ref()], bump)] 
    // pub nft_1_state: Account<'info, NftState>,
    // pub nft_2: AccountInfo<'info>,
    // #[account(seeds = [b"bapeBreeding".as_ref(), nft_2.key.as_ref()], bump)] 
    // pub nft_2_state: Account<'info, NftState>,

    pub baby_mint: AccountInfo<'info>, // mint for baby
    pub baby_nft: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct FastReact<'info> {
    pub user: Signer<'info>,
    #[account(mut, owner = *user.key)]
    pub potion: Account<'info, PotionState>,

    pub nft_1: AccountInfo<'info>,
    #[account(seeds = [PREFIX.as_bytes(), nft_1.key.as_ref()], bump)] 
    pub nft_1_state: Account<'info, NftState>,

    pub nft_2: AccountInfo<'info>,
    #[account(seeds = [PREFIX.as_bytes(), nft_2.key.as_ref()], bump)] 
    pub nft_2_state: Account<'info, NftState>,

    #[account(mut)]
    pub token_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,  // $BAPE mint, generic enough for any token though

    pub baby_mint: AccountInfo<'info>, // mint for baby
    pub baby_nft: AccountInfo<'info>,

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
}

#[error]
pub enum ErrorCode {
    #[msg("This NFT has been used for breeding in the last 10 days.")]
    NftUsedTooSoon,
    #[msg("This potion has not reached its cooldown period.")]
    CooldownNotReached,
    #[msg("User has insufficient funds to complete the transaction.")]
    InsufficientFunds,
    #[msg("User is not authorized to complete the transaction.")]
    Unauthorized,
    #[msg("NFT's do not match token provided.")]
    Mismatch,
    #[msg("Used wrong token.")]
    WrongToken,
    #[msg("Used same NFT's.")]
    SameNFTs,
}
