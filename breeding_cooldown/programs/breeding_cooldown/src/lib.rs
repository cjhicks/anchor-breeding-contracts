use anchor_lang::prelude::*;
use anchor_spl;
use solana_program::{
    sysvar::{clock::Clock, Sysvar, rent::Rent, self},
    system_instruction
};
use solana_program::program::{invoke_signed, invoke};
#[allow(unused_imports)]
use spl_token_metadata::{
        instruction::{update_metadata_accounts, CreateMetadataAccountArgs,CreateMasterEditionArgs, MetadataInstruction},
        state::{Creator, Data, Metadata, MAX_URI_LENGTH, MAX_NAME_LENGTH},
};
use solana_program::instruction::{Instruction,AccountMeta};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

fn get_timestamp() -> u64 {
    return Clock::get().unwrap().unix_timestamp as u64;
}

fn get_breed_min_timestamp(timestamp: u64) -> u64 {
    let seven_days_in_seconds = 7 * 24 * 60 * 60;
    return timestamp - seven_days_in_seconds;
}

#[program]
pub mod breeding_cooldown {
    use super::*;

    /*
    This function is equivalent to breeding an egg: https://explorer.solana.com/tx/g5fg51XveddE1MyU3GsEUpU6e3vUz1BhWNBvye6hBziDZbKsBv4H1UjLEKr1rjLFtABt6YNM6TBBoMzDxtQ5td5
    */
    pub fn create_potion(ctx: Context<CreatePotion>, authority: Pubkey) -> ProgramResult {
        // let CANDY_MACHINE_ID = "6vHjQxYUwk9DNuJNHfRWSfH1UTuikVayP9h3H4iYW2TD";

        let potion = &mut ctx.accounts.potion;
        let potion_mint = &mut ctx.accounts.potion_mint;
        let potion_mint_metadata_info = &mut ctx.accounts.potion_mint_metadata_info;
        // let program_id = &ctx.program_id;
        let token_program = &ctx.accounts.token_program;
        let token_metadata_program = &ctx.accounts.token_metadata_program;
        let user = &ctx.accounts.user;
        let token_mint = ctx.accounts.token_mint.to_account_info();
        let rent = &ctx.accounts.rent;
        let system_program = &ctx.accounts.system_program.to_account_info();
        let potion_master_edition = &ctx.accounts.potion_master_edition;

        /*
        Validation
        */
        // check token IS $bape
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

        // check if we have enough $BAPE before continuing
        let token_user_account = &ctx.accounts.token_user_account;
        let burn_price = 500;
        if token_user_account.amount < burn_price {
            return Err(ErrorCode::InsufficientFunds.into())
        }

        // check if 7 days since last breeding
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);

        let nft_1_metadata = &mut ctx.accounts.nft_1_metadata;
        if nft_1_metadata.last_bred_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }
        let nft_2_metadata = &mut ctx.accounts.nft_2_metadata;
        if nft_2_metadata.last_bred_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }

        /*
        Transfer
        */
        // TODO: I think data_account_info is potion data account?
        // invoke(
        //     &system_instruction::transfer(user.key, data_account_info.key, required_lamports),
        //     &[
        //         user.clone(),
        //         data_account_info.clone(),
        //         system_program.clone(),
        //     ],
        // )?;
        // invoke_signed(
        //     &system_instruction::allocate(data_account_info.key, size as u64),
        //     &[
        //         data_account_info.clone(),
        //         system_program.clone(),
        //     ],
        //     &[&[&"potion".as_bytes(),&mint_info.key.to_bytes(), &[data_bump]]],
        // )?;

        // invoke_signed(
        //     &system_instruction::assign(data_account_info.key, program_id),
        //     &[
        //         data_account_info.clone(),
        //         system_program.clone(),
        //     ],
        //     &[&[&"potion".as_bytes(),&mint_info.key.to_bytes(), &[data_bump]]],
        // )?;

        let nft_1_key = *ctx.accounts.nft_1.key;
        let nft_2_key = *ctx.accounts.nft_2.key;
        potion.authority = authority;
        potion.nft1 = nft_1_key;
        potion.nft2 = nft_2_key;
        potion.created_timestamp = timestamp;

        nft_1_metadata.authority = authority;
        nft_1_metadata.nft = nft_1_key;
        nft_1_metadata.last_bred_timestamp = timestamp;
        nft_2_metadata.authority = authority;
        nft_2_metadata.nft = nft_2_key;
        nft_2_metadata.last_bred_timestamp = timestamp;

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
        // let mut tmp = user.clone();
        // tmp.is_writable=false;
        // TODO: literally no idea what this is, copied it from tokenBreeding2
        let uri = r"https://bafybeibhsnl5sz32jdfdwfvj4qea3at25wobxenzwjdirrdr2h3i4u4y2a.ipfs.infura-ipfs.io";
        // TODO: what are these??? Just copied from bored ape creators for now...
        let mut creators_ptn = vec![
            Creator{
                address: "7CqaVHL7Wv6RzHoRDH4govgy38uUfj75UVgCLVwrKhus".parse::<Pubkey>().unwrap(),
                verified: false,
                share: 100,
            },
            Creator{
                address: "6vHjQxYUwk9DNuJNHfRWSfH1UTuikVayP9h3H4iYW2TD".parse::<Pubkey>().unwrap(),
                verified: false,
                share: 0,
            },
        ];
        let price_data_info_key = "7CqaVHL7Wv6RzHoRDH4govgy38uUfj75UVgCLVwrKhus".parse::<Pubkey>().unwrap(); // update authority? *price_data_info.key,
        let vault_creator = Creator{
            address: price_data_info_key,
            verified: true,
            share: 0,
        };
        creators_ptn.insert(0,vault_creator);
        let (price_address, price_bump) = Pubkey::find_program_address(&[&"price".as_bytes()], &ctx.program_id);
        invoke_signed(
            &create_metadata_accounts(
                spl_token_metadata::id(), 
                potion_mint_metadata_info.key(),
                potion_mint.key(),
                user.key(),
                user.key(),
                price_data_info_key,
                "Potion".to_string(),
                "PTN".to_string(),
                uri.to_string(),
                Some(creators_ptn),
                0, //royalties,
                true,
                true,
            ),
            &[  
                potion_mint_metadata_info.clone(),
                potion_mint.to_account_info(),
                user.to_account_info(),
                user.to_account_info(),
                // vault_creator.clone(),
                system_program.clone(),
                rent.clone(),
                token_metadata_program.to_account_info()
            ],
            &[&[&"price".as_bytes(), &[price_bump]]],
        )?;

        // let mut tmp1 =  price_data_info.clone();
        // tmp1.is_writable=false;
        // let mut tmp2 =  mint_metadata_info.clone();
        // tmp2.is_writable=false;
        invoke_signed(
            &create_master_edition(
                spl_token_metadata::id(), 
                potion_master_edition.key(),
                potion_mint.key(),
                price_data_info_key,
                user.key(),
                potion_mint_metadata_info.key(),
                user.key(),
                Some(0),
            ),
            &[  
                potion_master_edition.clone().to_account_info(),
                potion_mint.clone().to_account_info(),
                // price_data_info.clone().to_account_info(),
                user.clone().to_account_info(),
                user.clone().to_account_info(),
                potion_mint_metadata_info.clone().to_account_info(),
                potion.clone().to_account_info(),
                system_program.clone(),
                rent.clone(),
                token_metadata_program.to_account_info()
            ],
            &[&[&"price".as_bytes(), &[price_bump]]],
        )?;

        invoke_signed(
            &update_metadata_accounts(
                spl_token_metadata::id(), 
                potion_mint_metadata_info.key(),
                price_data_info_key,
                None,
                None,
                Some(true),
            ),
            &[  
                potion_mint_metadata_info.clone().to_account_info(),
                // price_data_info.clone(),
                token_metadata_program.to_account_info()
            ],
            &[&[&"price".as_bytes(), &[price_bump]]],
        )?;

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
        // let nft_1_metadata = &ctx.accounts.nft_1_metadata;
        // let nft_2_metadata = &ctx.accounts.nft_2_metadata;

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

        if potion.authority != user_key { //|| nft_1_metadata.authority != user_key || nft_2_metadata.authority != user_key {
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
        let nft_1_metadata = &ctx.accounts.nft_1_metadata;
        let nft_2_metadata = &ctx.accounts.nft_2_metadata;
        let token_program = &ctx.accounts.token_program;
        let user = &ctx.accounts.user;
        let token_mint = ctx.accounts.token_mint.to_account_info();

        if potion.authority != user_key || nft_1_metadata.authority != user_key || nft_2_metadata.authority != user_key {
            return Err(ErrorCode::Unauthorized.into());
        }

        // if !((potion.nft1 == nft_1_metadata.nft && potion.nft2 == nft_2_metadata.nft) ||
        //     (potion.nft1 == nft_2_metadata.nft && potion.nft2 == nft_1_metadata.nft)) {
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
pub struct Potion {
    pub authority: Pubkey,
    pub nft1: Pubkey,
    pub nft2: Pubkey,
    pub created_timestamp: u64
}

#[account]
pub struct NftMetadata {
    pub authority: Pubkey,
    pub nft: Pubkey,
    pub last_bred_timestamp: u64
}

#[derive(Accounts)]
pub struct CreatePotion<'info> {
    pub user: Signer<'info>,
    #[account(init, payer = user, space = 8 + 120)]
    pub potion: Account<'info, Potion>,
    // TODO: owner = user?
    #[account(mut)]
    pub token_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,  // $BAPE mint, generic enough for any token though
    // #[account(mut, owner = user)]
    // #[account(mut)]
    pub potion_mint: Account<'info, anchor_spl::token::Mint>, // mint for potions
    pub potion_mint_metadata_info: AccountInfo<'info>,
    #[account(mut)]
    pub potion_master_edition: AccountInfo<'info>,
    // TODO: owner is user
    // #[account(owner = *user.key)]
    pub nft_1: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [b"bapeBreeding".as_ref(), nft_1.key.as_ref()], bump, payer = user, space = 8 + 80)]
    pub nft_1_metadata: Account<'info, NftMetadata>,
    // #[account(owner = *user.key)]  -> Might need something else here
    pub nft_2: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [b"bapeBreeding".as_ref(), nft_2.key.as_ref()], bump, payer = user, space = 8 + 80)]    // pub nft_2_metadata: Account<'info, NftMetadata>,
    pub nft_2_metadata: Account<'info, NftMetadata>,
    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    pub token_metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: AccountInfo<'info>, // this just anchor.web3.SYSVAR_RENT_PUBKEY from frontend
}

#[derive(Accounts)]
pub struct React<'info> {
    pub user: Signer<'info>,
    #[account(mut, owner = *user.key)]
    pub potion: Account<'info, Potion>,

    // don't need this
    // pub nft_1: AccountInfo<'info>,
    // #[account(seeds = [b"bapeBreeding".as_ref(), nft_1.key.as_ref()], bump)] 
    // pub nft_1_metadata: Account<'info, NftMetadata>,
    // pub nft_2: AccountInfo<'info>,
    // #[account(seeds = [b"bapeBreeding".as_ref(), nft_2.key.as_ref()], bump)] 
    // pub nft_2_metadata: Account<'info, NftMetadata>,

    pub baby_mint: AccountInfo<'info>, // mint for baby
    pub baby_nft: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct FastReact<'info> {
    pub user: Signer<'info>,
    #[account(mut, owner = *user.key)]
    pub potion: Account<'info, Potion>,

    pub nft_1: AccountInfo<'info>,
    #[account(seeds = [b"bapeBreeding".as_ref(), nft_1.key.as_ref()], bump)] 
    pub nft_1_metadata: Account<'info, NftMetadata>,
    pub nft_2: AccountInfo<'info>,
    #[account(seeds = [b"bapeBreeding".as_ref(), nft_2.key.as_ref()], bump)] 
    pub nft_2_metadata: Account<'info, NftMetadata>,

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
    #[msg("This NFT has been used for breeding in the last 7 days.")]
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