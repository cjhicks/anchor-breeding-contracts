
use anchor_lang::prelude::*;
use anchor_spl;
use solana_program::{sysvar};
use solana_program::program::{invoke_signed};
use spl_token_metadata::{
        instruction::{update_metadata_accounts, CreateMetadataAccountArgs, CreateMasterEditionArgs, MetadataInstruction}, //create_metadata_accounts
        state::{Creator, Data}
};
use spl_token_metadata::state::{Metadata};
use anchor_lang::solana_program::program_pack::Pack;
use spl_token::state::Account as SplTokenAccount;
use solana_program::instruction::{Instruction,AccountMeta};
use std::{cell::RefMut, cell::RefCell};
use borsh::{BorshDeserialize, BorshSerialize};

declare_id!("9u5z6XtPcZaPSV4jM7NLpjdBgjZVVY4fa4J1mm2ZW2bD");

const PREFIX: &[u8] = b"bapeBrd7";
const PREFIX_POTION: &[u8] = b"ptn";
const PREFIX_MUTANT: &[u8] = b"mtnt";
const PREFIX_COUNT: &[u8] = b"cnt";

const CONFIG_ARRAY_START: usize = 8; // key
const MAX_URI_LENGTH: usize = 50;
const CONFIG_LINE_SIZE: usize = MAX_URI_LENGTH; // 4 + MAX_URI_LENGTH;

#[program]
pub mod breeding_cooldown {
    use super::*;

    pub fn init_uris(ctx: Context<InitUris>) -> ProgramResult {
        let uris = &mut ctx.accounts.uris;
        // uris.relative_uris = vec![];
        Ok(())
    }

    pub fn add_uri(ctx: Context<AddUri>, index: u16, relative_uri: String) -> ProgramResult {
        let uris = &mut ctx.accounts.uris;
        let account = uris.to_account_info();
        let mut data = account.data.borrow_mut();

        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_URI_LENGTH - relative_uri.len() {
            array_of_zeroes.push(0u8);
        }

        let uri = relative_uri.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        let as_vec = &uri.as_bytes().to_vec();
        let serialized: &[u8] = &as_vec.as_slice();//[4..];
        let position = CONFIG_ARRAY_START + 4 + (index as usize) * CONFIG_LINE_SIZE;
        let fixed_config_lines_len: usize = 1; // only adding one at a time
        let array_slice: &mut [u8] =
            &mut data[position..position + fixed_config_lines_len * CONFIG_LINE_SIZE];
        array_slice.copy_from_slice(serialized);

        Ok(())
    }

    pub fn create_potion(ctx: Context<CreatePotion>, creator_bump: u8) -> ProgramResult {
        let potion_mint = &mut ctx.accounts.potion_mint;
        let user = &ctx.accounts.user;
        let token_program = &ctx.accounts.token_program;

        // TODO: verify NFT is owned by wallet
        let ata_1 = SplTokenAccount::unpack(&ctx.accounts.nft_1_associated_token.data.borrow())?;
        let ata_2 = SplTokenAccount::unpack(&ctx.accounts.nft_2_associated_token.data.borrow())?;
        if ata_1.owner != *user.key || ata_2.owner != *user.key {
            return Err(ErrorCode::NftNotOwned.into())
        }
        // verify NFT is of BASC collection
        verify_collection(Metadata::from_account_info(&ctx.accounts.nft_1_metadata)?, *ctx.accounts.nft_1.key)?;
        verify_collection(Metadata::from_account_info(&ctx.accounts.nft_2_metadata)?, *ctx.accounts.nft_2.key)?;

        // check if 7 days since last breeding
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);
        let nft_1_state = &mut ctx.accounts.nft_1_state;
        let nft_2_state = &mut ctx.accounts.nft_2_state;
        if nft_1_state.last_bred_timestamp > breed_min_timestamp || nft_2_state.last_bred_timestamp > breed_min_timestamp {
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
        anchor_spl::token::burn(burn_ctx, 500 * 10_u64.pow(9))?;

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
            &[PREFIX, PREFIX_POTION, &[creator_bump]],
            potion_mint,
            &mut ctx.accounts.potion_mint_metadata,
            &ctx.accounts.potion_master_edition,
            &ctx.accounts.potion_token,
            token_program,
            &ctx.accounts.token_metadata_program,
            &ctx.accounts.system_program,
            &ctx.accounts.rent.to_account_info(),
            false
        )?;

        // ctx.accounts.potion_count.count += 1;

        Ok(())
    }

    pub fn react(ctx: Context<React>, creator_bump: u8) -> ProgramResult {
        // check if 10 days since last breeding
        let potion_state = &ctx.accounts.potion_state;
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);
        if potion_state.created_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }

        /*
        Burn potion
        */
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.clone(),
            anchor_spl::token::Burn {
                to: ctx.accounts.potion_token.to_account_info(),
                mint: ctx.accounts.potion_mint.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            }
        );
        anchor_spl::token::burn(burn_ctx, 1)
            .expect("burn failed.");

        let count = ctx.accounts.mutant_count.count;
        let name = format!("{}{}", "Mutant #", count + 1);
        let uri = format!("{}{}", r"https://arweave.net/", get_uri(&ctx.accounts.uris, count));
        mint_nft(
            name,
            "BASE".to_string(),
            uri,
            &ctx.accounts.user,
            &ctx.accounts.mutant_creator,
            &[PREFIX, PREFIX_MUTANT, &[creator_bump]],
            &ctx.accounts.mutant_mint,
            &mut ctx.accounts.mutant_mint_metadata,
            &ctx.accounts.mutant_master_edition,
            &ctx.accounts.mutant_token,
            &ctx.accounts.token_program,
            &ctx.accounts.token_metadata_program,
            &ctx.accounts.system_program,
            &ctx.accounts.rent.to_account_info(),
            true
        )?;

        ctx.accounts.mutant_count.count += 1;

        Ok(())
    }

    pub fn fast_react(ctx: Context<FastReact>, creator_bump: u8) -> ProgramResult {
        let token_program = &ctx.accounts.token_program;
        let user = &ctx.accounts.user;
        let token_mint = ctx.accounts.token_mint.to_account_info();

        let token_user_account = &ctx.accounts.token_user_account;
        let fast_burn_price = 250 * 10_u64.pow(9);
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

        /*
        Burn potion
        */
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.clone(),
            anchor_spl::token::Burn {
                to: ctx.accounts.potion_token.to_account_info(),
                mint: ctx.accounts.potion_mint.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            }
        );
        anchor_spl::token::burn(burn_ctx, 1)
            .expect("burn failed.");

        let count = ctx.accounts.mutant_count.count;
        let name = format!("{}{}", "Mutant #", count + 1);
        let uri = format!("{}{}", r"https://arweave.net/", get_uri(&ctx.accounts.uris, count));
        mint_nft(
            name,
            "BASE".to_string(),
            uri,
            &ctx.accounts.user,
            &ctx.accounts.mutant_creator,
            &[PREFIX, PREFIX_MUTANT, &[creator_bump]],
            &ctx.accounts.mutant_mint,
            &mut ctx.accounts.mutant_mint_metadata,
            &ctx.accounts.mutant_master_edition,
            &ctx.accounts.mutant_token,
            &ctx.accounts.token_program,
            &ctx.accounts.token_metadata_program,
            &ctx.accounts.system_program,
            &ctx.accounts.rent.to_account_info(),
            true
        )?;

        ctx.accounts.mutant_count.count += 1;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct CreatePotion<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
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


    // TODO: figure out seeds::program so we can assert update_authority for collection. 
    // right now, we're getting the "account owned different program" error
    pub nft_1: AccountInfo<'info>,
    // #[account(
    //     seeds = [b"metadata", token_metadata_program.key().as_ref(), nft_1.key.as_ref()],
    //     bump,
    //     seeds::program = token_metadata_program.key()
    // )]
    pub nft_1_associated_token: AccountInfo<'info>,
    pub nft_1_metadata: AccountInfo<'info>, //<'info, Metadata>,
    // TODO: come back for validations
    // constraint= config.to_account_info().owner
    #[account(init_if_needed, seeds = [PREFIX, nft_1.key.as_ref()], bump, payer = user, space = 8 + 40)]
    pub nft_1_state: Account<'info, NftState>,
    // // #[account(owner = *user.key)]
    #[account(constraint = nft_2.key() != nft_1.key() @ ErrorCode::SameNFTs)]
    pub nft_2: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [PREFIX, nft_2.key.as_ref()], bump, payer = user, space = 8 + 40)]
    pub nft_2_state: Account<'info, NftState>,
    pub nft_2_metadata: AccountInfo<'info>,
    // #[account(
    //     seeds = [b"metadata", token_metadata_program.key().as_ref(), nft_1.key.as_ref()],
    //     bump,
    //     seeds::program = token_metadata_program.key()
    // )]
    pub nft_2_associated_token: AccountInfo<'info>,

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    // #[account(address = .as_ref())]
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
    #[account(mut)]
    pub potion_token: AccountInfo<'info>,
    #[account(seeds = [PREFIX.as_ref(), potion_mint.key.as_ref()], bump)]
    pub potion_state: Account<'info, PotionState>,
    #[account(
        init_if_needed,
        seeds = [PREFIX.as_ref(), PREFIX_MUTANT, PREFIX_COUNT.as_ref()], bump, payer = user, space = 8 + 30,
        constraint = mutant_count.count < (3333 as u16) @ ErrorCode::NoMoreMutants
    )]
    pub mutant_count: Account<'info, Counter>,
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
    #[account(mut, constraint = uris.to_account_info().owner == program_id)]
    pub uris: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct FastReact<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub potion_mint: AccountInfo<'info>,
    // TODO: creator check?
    #[account(mut)]
    pub potion_token: AccountInfo<'info>,
    #[account(seeds = [PREFIX.as_ref(), potion_mint.key.as_ref()], bump)]
    pub potion_state: Account<'info, PotionState>,
    #[account(mut)]
    pub token_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    #[account(
        mut,
        constraint = token_mint.key() == "2RTsdGVkWJU7DG77ayYTCvZctUVz3L9Crp9vkMDdRt4Y".parse::<Pubkey>().unwrap() @ ErrorCode::WrongToken
    )]
    pub token_mint: AccountInfo<'info>,  // $BAPE mint, generic enough for any token though
    #[account(
        init_if_needed,
        seeds = [PREFIX.as_ref(), PREFIX_MUTANT, PREFIX_COUNT.as_ref()], bump, payer = user, space = 8 + 30,
        constraint = mutant_count.count < (3333 as u16) @ ErrorCode::NoMoreMutants
    )]
    pub mutant_count: Account<'info, Counter>,
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
    #[account(mut, constraint = uris.to_account_info().owner == program_id)]
    pub uris: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction()]
pub struct InitUris<'info> {
    // #[account(mut)]
    pub user: Signer<'info>,
    // #[account(init, seeds = [PREFIX, PREFIX_URI], bump, payer = user, space = 8 + 10000)]
    // pub uris: Account<'info, Uris>,
    #[account(zero, rent_exempt = skip, constraint = uris.to_account_info().owner == program_id && uris.to_account_info().data_len() >= get_space_for_uris()?)]
    pub uris: UncheckedAccount<'info>,

    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(index: u16, relative_uri: String)]
pub struct AddUri<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = uris.to_account_info().owner == program_id)]  // , seeds = [PREFIX, PREFIX_URI], bump
    pub uris: AccountInfo<'info>, //<'info, Uris>,
    pub system_program: AccountInfo<'info>
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
    #[msg("Invalid String")]
    InvalidString,
    #[msg("No Creators")]
    NoCreators,
    #[msg("Wrong Creator")]
    WrongCreator,
    #[msg("Unverified Creator")]
    UnverifiedCreator,
    #[msg("Wrong Update Authority")]
    WrongUpdateAuthority,
    #[msg("Wrong NFT")]
    WrongNft,
    #[msg("User is not owner of NFT")]
    NftNotOwned
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

#[account]
#[derive(Default)]
pub struct Counter {
    pub count: u16
}

fn get_timestamp() -> u64 {
    return Clock::get().unwrap().unix_timestamp as u64;
}

fn get_breed_min_timestamp(timestamp: u64) -> u64 {
    let ten_days_in_seconds = 10 * 24 * 60 * 60;
    return timestamp - ten_days_in_seconds;
}

fn get_space_for_uris() -> core::result::Result<usize, ProgramError> {
    let items_available = 3333;
    let num = CONFIG_ARRAY_START
            + 4
            + (items_available as usize) * CONFIG_LINE_SIZE
            + 8;
    Ok(num)
}

fn get_uri<'a>(uris: &AccountInfo<'a>, index: u16) -> String {
    let account = uris.to_account_info();
    let mut arr = account.data.borrow_mut();

    let data_array = &mut arr[CONFIG_ARRAY_START + 4 + (index as usize) * (CONFIG_LINE_SIZE)
    ..CONFIG_ARRAY_START + 4 + ((index as usize) + 1) * (CONFIG_LINE_SIZE)];

    let mut uri_vec = vec![];
    for i in 0..MAX_URI_LENGTH {
        if data_array[i] != 0u8 {
            uri_vec.push(data_array[i]);
        }
    }
    return String::from_utf8(uri_vec).unwrap();
}

fn verify_collection(metadata: Metadata, nft: Pubkey) -> ProgramResult {
    let nft_update_authority = "7CqaVHL7Wv6RzHoRDH4govgy38uUfj75UVgCLVwrKhus".parse::<Pubkey>().unwrap(); // TODO: 4dKSgRptpvveQ73kJvzw88gF7YPs4hoWfrJnzBhbmi1i
    let creator_0_key = "6vHjQxYUwk9DNuJNHfRWSfH1UTuikVayP9h3H4iYW2TD".parse::<Pubkey>().unwrap(); // TODO: 4SRNmDuitWA1fZfg72WSThoKd2ENEnQeo4NFPcn3xunf
    match metadata.data.creators {
        None => { return Err(ErrorCode::NoCreators.into()) }
        Some(creators) => {
            match creators.first() {
                None => { return Err(ErrorCode::NoCreators.into()) }
                Some(creator) => {
                    if metadata.update_authority != nft_update_authority {
                        return Err(ErrorCode::WrongUpdateAuthority.into());
                    } else if metadata.mint != nft { 
                        return Err(ErrorCode::WrongNft.into());
                    } else if creator.address != creator_0_key {
                        return Err(ErrorCode::WrongCreator.into());
                    } else if !creator.verified {
                        return Err(ErrorCode::UnverifiedCreator.into());
                    }  else {
                        return Ok(());
                    }
                }
            }
        }
    };
}

pub fn mint_nft<'a>(
    name: String,
    symbol: String,
    uri: String,
    user: &Signer<'a>,
    creator: &AccountInfo<'a>,
    creator_seeds: &[&[u8]],
    mint: &AccountInfo<'a>,
    mint_metadata: &AccountInfo<'a>,
    master_edition: &AccountInfo<'a>,
    token: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    token_metadata_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    user_is_creator: bool
) -> ProgramResult {
    let creators_ptn = match user_is_creator {
        true => {
            vec![
                Creator{
                    address: creator.key(),
                    verified: true,
                    share: 0,
                },
                Creator{
                    address: *user.key,
                    verified: false,
                    share: 20,
                },
                Creator{
                    address: "4dKSgRptpvveQ73kJvzw88gF7YPs4hoWfrJnzBhbmi1i".parse::<Pubkey>().unwrap(),
                    verified: false,
                    share: 80,
                },
            ]
        }
        false => vec![
            Creator{
                address: creator.key(),
                verified: true,
                share: 0,
            },
            Creator{
                address: "4dKSgRptpvveQ73kJvzw88gF7YPs4hoWfrJnzBhbmi1i".parse::<Pubkey>().unwrap(),
                verified: false,
                share: 100,
            },
        ]
    };

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
        &[creator_seeds]
    )?;

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
        &[creator_seeds]
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
        &[creator_seeds]
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
