
use anchor_lang::prelude::*;
use anchor_spl;
use solana_program::{sysvar};
use solana_program::program::{invoke_signed};
use spl_token_metadata::{
        instruction::{update_metadata_accounts, CreateMetadataAccountArgs, CreateMasterEditionArgs, MetadataInstruction}, //create_metadata_accounts
        state::{Creator, Data},
};
use solana_program::instruction::{Instruction,AccountMeta};


declare_id!("9CNNoWiwBJzQzW72ycRvZyFQLqkyiN4TkmzmNooiTBsw");

const PREFIX: &[u8] = b"bapeBrd2";
const PREFIX_POTION: &[u8] = b"ptn";
// const PREFIX_COUNT: &[u8] = b"cnt";
const PREFIX_MUTANT: &[u8] = b"mtnt";

#[program]
pub mod breeding_cooldown {
    use super::*;

    // TODO: check NFT is of BASC collection
    pub fn create_potion(ctx: Context<CreatePotion>, creator_bump: u8) -> ProgramResult {
        let potion_mint = &mut ctx.accounts.potion_mint;
        let user = &ctx.accounts.user;
        let token_program = &ctx.accounts.token_program;

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

        // set state
        ctx.accounts.potion_state.created_timestamp = timestamp;
        nft_1_state.last_bred_timestamp = timestamp;
        nft_2_state.last_bred_timestamp = timestamp;

        /*
        Burn $BAPE after minting potion
        */
        let burn_ctx = CpiContext::new(
            token_program.clone(),
            anchor_spl::token::Burn {
                to: ctx.accounts.token_user_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                authority: user.to_account_info(),
            }
        );
        anchor_spl::token::burn(burn_ctx, 500 * 10_u64.pow(9))
            .expect("burn failed.");

        /* 
        Mint new NFT for potion
        */
        let uri = r"https://arweave.net/OEbN9FS8F4_P7nj_WoWoXuaour_oN4BVSZRbxrXTStc";
        mint_nft(
            "Protocol #367".to_string(),
            "BASE".to_string(),
            uri.to_string(),
            user,
            &ctx.accounts.potion_creator,
            creator_bump,
            potion_mint,
            &mut ctx.accounts.potion_mint_metadata,
            &ctx.accounts.potion_master_edition,
            &ctx.accounts.potion_token,
            token_program,
            &ctx.accounts.token_metadata_program,
            &ctx.accounts.system_program,
            &ctx.accounts.rent.to_account_info()
        )?;

        // ctx.accounts.potion_count.count += 1;

        Ok(())
    }

    pub fn react(ctx: Context<React>, creator_bump: u8) -> ProgramResult {
        // TODO: check potion is of collection

        // check if 7 days since last breeding
        let potion_state = &ctx.accounts.potion_state;
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);
        // if potion_state.created_timestamp > breed_min_timestamp {
        //     return Err(ErrorCode::NftUsedTooSoon.into());
        // }

        // TODO: get custom URI for this mutant!!!
        let uri = r"https://arweave.net/OEbN9FS8F4_P7nj_WoWoXuaour_oN4BVSZRbxrXTStc";
        mint_nft(
            "Mutant #367".to_string(),
            "BASE".to_string(),
            uri.to_string(),
            &ctx.accounts.user,
            &ctx.accounts.mutant_creator,
            creator_bump,
            &ctx.accounts.mutant_mint,
            &mut ctx.accounts.mutant_mint_metadata,
            &ctx.accounts.mutant_master_edition,
            &ctx.accounts.mutant_token,
            &ctx.accounts.token_program,
            &ctx.accounts.token_metadata_program,
            &ctx.accounts.system_program,
            &ctx.accounts.rent.to_account_info()
        )?;

        // ctx.accounts.mutant_count.count += 1;

        Ok(())
    }

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
    // #[account(
    //     init_if_needed,
    //     seeds = [PREFIX.as_ref(), PREFIX_COUNT.as_ref()], bump, payer = user, space = 8 + 30,
    //     constraint = potion_count.count < (3333 as u16) @ ErrorCode::NoMorePotions
    // )]
    // pub potion_count: Account<'info, Counter>,
    #[account(mut)]
    pub potion_mint: AccountInfo<'info>,
    #[account(init, seeds = [PREFIX.as_ref(), potion_mint.key.as_ref()], bump, payer = user, space = 8 + 80)]
    pub potion_state: Account<'info, PotionState>,
    #[account(mut, seeds = [PREFIX.as_ref(), PREFIX_POTION.as_ref()], bump=creator_bump)]
    pub potion_creator: AccountInfo<'info>,
    #[account(mut)]
    pub potion_mint_metadata: AccountInfo<'info>,
    #[account(mut)]
    pub potion_master_edition: AccountInfo<'info>,
    #[account(mut)]
    pub potion_token: AccountInfo<'info>,
    // TODO: owner = user?
    #[account(mut)]
    pub token_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    #[account(
        mut,
        constraint = token_mint.key() == "2RTsdGVkWJU7DG77ayYTCvZctUVz3L9Crp9vkMDdRt4Y".parse::<Pubkey>().unwrap() @ ErrorCode::WrongToken
    )]
    pub token_mint: AccountInfo<'info>,  // $BAPE mint, generic enough for any token though
    // #[account(owner = *user.key)]
    pub nft_1: AccountInfo<'info>,
    // TODO: come back for validations
    // constraint= config.to_account_info().owner
    #[account(init_if_needed, seeds = [PREFIX, nft_1.key.as_ref()], bump, payer = user, space = 8 + 40)]
    pub nft_1_state: Account<'info, NftState>,
    // // #[account(owner = *user.key)]
    #[account(constraint = nft_2.key() != nft_1.key() @ ErrorCode::SameNFTs)]
    pub nft_2: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [PREFIX, nft_2.key.as_ref()], bump, payer = user, space = 8 + 40)]
    pub nft_2_state: Account<'info, NftState>,

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    // #[account(address = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s".as_ref())]
    pub token_metadata_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct React<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub potion_mint: AccountInfo<'info>,
    #[account(seeds = [PREFIX.as_ref(), potion_mint.key.as_ref()], bump)]
    pub potion_state: Account<'info, PotionState>,
    // #[account(
    //     init_if_needed,
    //     seeds = [PREFIX.as_ref(), PREFIX_MUTANT, PREFIX_COUNT.as_ref()], bump, payer = user, space = 8 + 30,
    //     constraint = mutant_count.count < (3333 as u16) @ ErrorCode::NoMoreMutants
    // )]
    // pub mutant_count: Account<'info, Counter>,
    #[account(mut)]
    pub mutant_mint: AccountInfo<'info>,
    #[account(mut, seeds = [PREFIX, PREFIX_MUTANT], bump=creator_bump)]
    pub mutant_creator: AccountInfo<'info>,
    #[account(mut)]
    pub mutant_mint_metadata: AccountInfo<'info>,
    #[account(mut)]
    pub mutant_master_edition: AccountInfo<'info>,
    #[account(mut)]
    pub mutant_token: AccountInfo<'info>,

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    // #[account(address = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s".as_ref())]
    pub token_metadata_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>, // this is just anchor.web3.SystemProgram.programId from frontend
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
    #[msg("No more mutants available.")]
    NoMoreMutants,
}

fn get_timestamp() -> u64 {
    return Clock::get().unwrap().unix_timestamp as u64;
}

fn get_breed_min_timestamp(timestamp: u64) -> u64 {
    let ten_days_in_seconds = 10 * 24 * 60 * 60;
    return timestamp - ten_days_in_seconds;
}

pub fn mint_nft<'a>(
    name: String,
    symbol: String,
    uri: String,
    user: &Signer<'a>,
    creator: &AccountInfo<'a>,
    creator_bump: u8,
    mint: &AccountInfo<'a>,
    mint_metadata: &AccountInfo<'a>,
    master_edition: &AccountInfo<'a>,
    token: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    token_metadata_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
) -> ProgramResult {
    let creators_ptn = vec![
        Creator{
            address: creator.key(),
            verified: true,
            share: 0,
        },
        Creator{
            address: "4dKSgRptpvveQ73kJvzw88gF7YPs4hoWfrJnzBhbmi1i".parse::<Pubkey>().unwrap().key(),
            verified: false,
            share: 100,
        },
    ];

    invoke_signed(
        &create_metadata_accounts(    
            *token_metadata_program.key,
            *mint_metadata.key,
            *mint.key,
            *user.key,
            *user.key,
            *creator.key,
            name,
            symbol,
            uri,
            Some(creators_ptn),
            500, //royalties,
            true,
            true,
        ),
        &[
            mint_metadata.clone(),
            mint.to_account_info(),
            user.to_account_info(),
            creator.clone(),
            token_program.to_account_info(),
            system_program.clone(),
            rent.clone(),
            token_metadata_program.to_account_info()
        ],
        &[&[PREFIX, PREFIX_POTION, &[creator_bump]]]
    ).expect("create_metadata_accounts failed.");

    invoke_signed(
        &create_master_edition(
            *token_metadata_program.key, 
            *master_edition.key,
            *mint.key,
            *creator.key,
            *user.key,
            *mint_metadata.key,
            *user.key,
            Some(0),
        ),
        &[  
            master_edition.to_account_info(),
            mint.to_account_info(),
            creator.clone(),
            user.to_account_info(),
            mint_metadata.to_account_info(),
            token.to_account_info(),
            system_program.clone(),
            rent.clone(),
            token_metadata_program.to_account_info()
        ],
        &[&[PREFIX, PREFIX_POTION, &[creator_bump]]]
    )?;

    invoke_signed(
        &update_metadata_accounts(
            *token_metadata_program.key, 
            *mint_metadata.key,
            *creator.key,
            None,
            None,
            Some(true),
        ),
        &[  
            mint_metadata.to_account_info(),
            creator.clone(),
            token_metadata_program.to_account_info()
        ],
        &[&[PREFIX, PREFIX_POTION, &[creator_bump]]]
    )?;

    Ok(())
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
    pub created_timestamp: u64
}

#[account]
#[derive(Default)]
pub struct NftState {
    pub last_bred_timestamp: u64
}

// #[account]
// #[derive(Default)]
// pub struct Counter {
//     pub count: u16
// }