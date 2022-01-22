
use anchor_lang::prelude::*;
use anchor_spl;
use solana_program::{sysvar};
use solana_program::program::{invoke_signed};
use spl_token_metadata::{
        instruction::{update_metadata_accounts, CreateMetadataAccountArgs, CreateMasterEditionArgs, MetadataInstruction}, //create_metadata_accounts
        state::{Creator, Data},
};
use solana_program::instruction::{Instruction,AccountMeta};


declare_id!("Ajg8yy4gNuLwMWdH1k7sWVNaZb3nMu4wMHY8YED4iY6Y");

const PREFIX: &str = "bapeBreedingTest17";
const PREFIX_POTION: &str = "potion";

#[program]
pub mod breeding_cooldown {
    use super::*;
    /*
    This function is equivalent to breeding an egg: https://explorer.solana.com/tx/g5fg51XveddE1MyU3GsEUpU6e3vUz1BhWNBvye6hBziDZbKsBv4H1UjLEKr1rjLFtABt6YNM6TBBoMzDxtQ5td5
    */
    pub fn create_potion(ctx: Context<CreatePotion>, creator_bump: u8) -> ProgramResult {
        let potion_mint = &mut ctx.accounts.potion_mint;
        let potion_mint_metadata = &mut ctx.accounts.potion_mint_metadata;
        let potion_master_edition = &ctx.accounts.potion_master_edition;
        let potion_token = &ctx.accounts.potion_token;
        let potion_creator = &ctx.accounts.potion_creator;
        let other_creator = &ctx.accounts.other_creator;
        let nft_1 = &ctx.accounts.nft_1;
        let nft_2 = &ctx.accounts.nft_2;

        let user = &ctx.accounts.user;
        let token_mint = ctx.accounts.token_mint.to_account_info();

        let token_program = &ctx.accounts.token_program;
        let token_metadata_program = &ctx.accounts.token_metadata_program;
        let system_program = &ctx.accounts.system_program.to_account_info();
        let rent = &ctx.accounts.rent;

        // check global potion count
        let potion_count = &mut ctx.accounts.potion_count;
        if potion_count.count >= (3333 as u16) {
            return Err(ErrorCode::NoMorePotions.into())
        }

        // check token is $bape - change for prod
        let bape_mint = "2RTsdGVkWJU7DG77ayYTCvZctUVz3L9Crp9vkMDdRt4Y".parse::<Pubkey>().unwrap();
        if token_mint.key() != bape_mint {
            return Err(ErrorCode::WrongToken.into())
        }

        // check NFT's are not same
        if nft_1.key() == nft_2.key() {
            return Err(ErrorCode::SameNFTs.into())
        }

        // // check if 7 days since last breeding
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

        // check if we have enough $BAPE before continuing
        let token_user_account = &ctx.accounts.token_user_account;
        let decimals = 9;
        let base: u64 = 10;
        let burn_price = 500 * base.pow(decimals);
        if token_user_account.amount < burn_price {
            return Err(ErrorCode::InsufficientFunds.into())
        }

        // set state
        let potion_state = &mut ctx.accounts.potion_state;
        potion_state.nft1 = *ctx.accounts.nft_1.key;
        potion_state.nft2 = *ctx.accounts.nft_2.key;
        potion_state.created_timestamp = timestamp;
        nft_1_state.last_bred_timestamp = timestamp;
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
        let uri = r"https://arweave.net/OEbN9FS8F4_P7nj_WoWoXuaour_oN4BVSZRbxrXTStc";
        let creators_ptn = vec![
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
            *token_metadata_program.key,// spl_token_metadata::id(), 
            *potion_mint_metadata.key,
            *potion_mint.key,
            *user.key,
            *user.key,
            *potion_creator.key,
            "Protocol #367".to_string(),
            "BASE".to_string(),
            uri.to_string(),
            Some(creators_ptn),
            500, //royalties,
            true,
            true, // false?
        );
        invoke_signed(
            create_metadata_ix,
            &[
                potion_mint_metadata.clone(),
                potion_mint.to_account_info().clone(),
                user.to_account_info().clone(),
                potion_creator.clone(),
                token_program.to_account_info().clone(),
                system_program.clone(),
                rent.to_account_info().clone(),
                token_metadata_program.to_account_info().clone()
            ],
            &[&[PREFIX.as_bytes(), PREFIX_POTION.as_bytes(), &[creator_bump]]]
        ).expect("create_metadata_accounts failed.");

        invoke_signed(
            &create_master_edition(
                token_metadata_program.key(), 
                potion_master_edition.key(),
                potion_mint.key(),
                potion_creator.key(),
                user.key(),
                potion_mint_metadata.key(),
                user.key(),
                Some(0),
            ),
            &[  
                potion_master_edition.clone().to_account_info(),
                potion_mint.clone().to_account_info(),
                potion_creator.clone(),
                user.clone().to_account_info(),
                potion_mint_metadata.clone().to_account_info(),
                potion_token.clone().to_account_info(),
                system_program.clone(),
                rent.to_account_info().clone(),
                token_metadata_program.to_account_info()
            ],
            &[&[PREFIX.as_bytes(), PREFIX_POTION.as_bytes(), &[creator_bump]]]
        )?;

        invoke_signed(
            &update_metadata_accounts(
                token_metadata_program.key(), 
                *potion_mint_metadata.key,
                *potion_creator.key,
                None,
                None,
                Some(true),
            ),
            &[  
                potion_mint_metadata.clone().to_account_info(),
                potion_creator.clone(),
                token_metadata_program.to_account_info()
            ],
            &[&[PREFIX.as_bytes(), PREFIX_POTION.as_bytes(), &[creator_bump]]]
        )?;

        Ok(())
    }

    // pub fn react(ctx: Context<React>) -> ProgramResult {

    //     let timestamp = get_timestamp();
    //     let breed_min_timestamp = get_breed_min_timestamp(timestamp);

    //     /*
    //     Validations (function)
    //     1. User is Authority on Potion
    //     2. NFT metadata matches Potion and Authority
    //     3. Created Timestamp > 7 days
    //     4. Verify Mint on egg is legit
    //     */
    //     let user_key = *ctx.accounts.user.key;
    //     let potion = &mut ctx.accounts.potion;
    //     // let nft_1_state = &ctx.accounts.nft_1_state;
    //     // let nft_2_state = &ctx.accounts.nft_2_state;

    //     // TODO: Token Owner is me
    //     // TODO: Harcoded Potion Mint ID?

    //     // TODO: mint before hand? init mint?
    //     // TODO: Candy machine with same image
    //     // Update authority - one created (wallet)

    //     // mint authority - one that minted it - can't create?
    //     // on egg, mint authority is metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s - Metaplex Token Metadata
    //     // https://solscan.io/token/DjcECAj4TYJANgr9oFmcZTcdJrA5icnoscs5k3CddbVS

    //     // TODO: createAssociatedTokenAccountInstruction(potionToken, walletKey, walletKey, potionMint),
    //     // TODO: Transfer Potion

    //     if potion.authority != user_key { //|| nft_1_state.authority != user_key || nft_2_state.authority != user_key {
    //         return Err(ErrorCode::Unauthorized.into());
    //     }

    //     if potion.created_timestamp > breed_min_timestamp {
    //         return Err(ErrorCode::CooldownNotReached.into());
    //     }

    //     // TODO: mint new NFT (master edition)
    //     // TODO: make this a reusable function
    //     // TODO: for now, maybe follow this? Then hook into candy machine later: https://spl.solana.com/token#example-create-a-non-fungible-token

    //     // TODO: Create baby
    //     // client: use PublicKey.findProgramAddress to create empty (existing) address (state) for each NFT input
    //     // server: verify egg over 7 days - no parent necessary?
    //     // - or, should we require pass it in to verify its still held?

    //     // FIrst Instructions
    //     // 1 - Create Account (walletKey, new=babyMint)
    //     // 2 - Init Mint (walletKey, mint: babyMint)
    //     // 3 - Associated Token Create (account=babyToken, mint=babyMint, walletKey)
    //     // 4 - MintTo (token=babyMint, account=babyToken, mint=BabyMint, authority=walletKey)

    //     // Unknown Program Instructions

    //     Ok(())
    // }

    // pub fn fast_react(ctx: Context<FastReact>) -> ProgramResult {
    //     /*
    //     Validations (function)
    //     1. User is Authority on Potion
    //     2. NFT metadata matches Potion and Authority
    //     3. User has enough $BAPE
    //     4. Verify Mint on egg is legit
    //     */
    //     let user_key = *ctx.accounts.user.key;
    //     let potion = &mut ctx.accounts.potion;
    //     let nft_1_state = &ctx.accounts.nft_1_state;
    //     let nft_2_state = &ctx.accounts.nft_2_state;
    //     let token_program = &ctx.accounts.token_program;
    //     let user = &ctx.accounts.user;
    //     let token_mint = ctx.accounts.token_mint.to_account_info();

    //     if potion.authority != user_key || nft_1_state.authority != user_key || nft_2_state.authority != user_key {
    //         return Err(ErrorCode::Unauthorized.into());
    //     }

    //     // if !((potion.nft1 == nft_1_state.nft && potion.nft2 == nft_2_state.nft) ||
    //     //     (potion.nft1 == nft_2_state.nft && potion.nft2 == nft_1_state.nft)) {
    //     //     return Err(ErrorCode::Mismatch.into());
    //     // }
    //     // TODO: do fast reaction (burn more $BAPE?)
    //     let token_user_account = &ctx.accounts.token_user_account;
    //     let fast_burn_price = 250;
    //     if token_user_account.amount < fast_burn_price {
    //         return Err(ErrorCode::InsufficientFunds.into())
    //     }

    //     // TODO: mint new NFT
    //     // anchor_spl::token::transfer(ctx: CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>, amount: u64)

    //     // TODO: after mint successful, burn 175 $BAPE
    //     let burn_ctx = CpiContext::new(
    //         token_program.clone(),
    //         anchor_spl::token::Burn {
    //             to: token_user_account.to_account_info(),
    //             mint: token_mint,
    //             authority: user.to_account_info(),
    //         }
    //     );
    //     anchor_spl::token::burn(burn_ctx, fast_burn_price)
    //         .expect("burn failed.");

    //     Ok(())
    // }
}

#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct CreatePotion<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, seeds = [PREFIX.as_ref(), potion_mint.key.as_ref()], bump, payer = user, space = 8 + 20)]
    pub potion_count: Account<'info, PotionCount>,
    #[account(mut)]
    pub potion_mint: AccountInfo<'info>,
    #[account(init, seeds = [PREFIX.as_ref(), potion_mint.key.as_ref()], bump, payer = user, space = 8 + 80)]
    pub potion_state: Account<'info, PotionState>,
    #[account(mut, seeds = [PREFIX.as_ref(), PREFIX_POTION.as_ref()], bump=creator_bump)]
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
    #[account(init_if_needed, seeds = [PREFIX.as_bytes(), nft_1.key.as_ref()], bump, payer = user, space = 8 + 40)]
    pub nft_1_state: Account<'info, NftState>,
    // // #[account(owner = *user.key)]
    pub nft_2: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [PREFIX.as_bytes(), nft_2.key.as_ref()], bump, payer = user, space = 8 + 40)]
    pub nft_2_state: Account<'info, NftState>,

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    // #[account(address = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s".as_ref())]
    pub token_metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: Sysvar<'info, Rent>,
}

#[error]
pub enum ErrorCode {
    #[msg("No more potions available.")]
    NoMorePotions,
    #[msg("Used wrong token.")]
    WrongToken,
    #[msg("Used same NFT's.")]
    SameNFTs,
    #[msg("This NFT has been used for breeding in the last 10 days.")]
    NftUsedTooSoon,
    #[msg("User has insufficient funds to complete the transaction.")]
    InsufficientFunds,
}

fn get_timestamp() -> u64 {
    return Clock::get().unwrap().unix_timestamp as u64;
}

fn get_breed_min_timestamp(timestamp: u64) -> u64 {
    let seven_days_in_seconds = 7 * 24 * 60 * 60;
    return timestamp - seven_days_in_seconds;
}

pub fn create_metadata_accounts(
    program_id: Pubkey,
    metadata_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>,
    seller_fee_basis_points: u16,
    update_authority_is_signer: bool,
    is_mutable: bool,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(update_authority, update_authority_is_signer),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::CreateMetadataAccount(CreateMetadataAccountArgs {
            data: Data {
                name,
                symbol,
                uri,
                seller_fee_basis_points,
                creators,
            },
            is_mutable,
        })
        .try_to_vec()
        .unwrap(),
    }
}

pub fn create_master_edition(
    program_id: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
    update_authority: Pubkey,
    mint_authority: Pubkey,
    metadata: Pubkey,
    payer: Pubkey,
    max_supply: Option<u64>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(edition, false),
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(update_authority, true),
        AccountMeta::new_readonly(mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::CreateMasterEdition(CreateMasterEditionArgs { max_supply })
            .try_to_vec()
            .unwrap(),
    }
}

#[account]
#[derive(Default)]
pub struct PotionState {
    pub nft1: Pubkey,
    pub nft2: Pubkey,
    pub created_timestamp: u64
}

#[account]
#[derive(Default)]
pub struct NftState {
    pub last_bred_timestamp: u64
}

#[account]
#[derive(Default)]
pub struct PotionCount {
    pub count: u16
}