use anchor_lang::prelude::*;
use anchor_spl;
use solana_program::{sysvar};
use solana_program::program::{invoke_signed};
use spl_token_metadata::{
        instruction::{update_metadata_accounts, CreateMetadataAccountArgs, CreateMasterEditionArgs, MetadataInstruction}, //create_metadata_accounts
        state::{Creator, Data},
};
use solana_program::instruction::{Instruction,AccountMeta};

declare_id!("G23nNQ3tMzXaT4SQecJfW6JSwuMhsBruU9iZvhPFeehu");

const PREFIX: &str = "bapeBreedingTest17";
const PREFIX_POTION: &str = "potion";

#[program]
pub mod breeding_state {
    use super::*;

    pub fn update_state(ctx: Context<UpdateState>) -> ProgramResult {
        let user = &ctx.accounts.user;
        let token_mint = ctx.accounts.token_mint.to_account_info();
        let token_program = &ctx.accounts.token_program;

        // check if we have enough $BAPE before continuing
        let token_user_account = &ctx.accounts.token_user_account;
        let decimals = 9;
        let base: u64 = 10;
        let burn_price = 500 * base.pow(decimals);
        if token_user_account.amount < burn_price {
            return Err(ErrorCode::InsufficientFunds.into())
        }

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

        Ok(())
    }

    pub fn mint_potion(ctx: Context<MintPotion>, creator_bump: u8) -> ProgramResult {
        let potion_mint = &mut ctx.accounts.potion_mint;
        let potion_mint_metadata = &mut ctx.accounts.potion_mint_metadata;
        let potion_master_edition = &ctx.accounts.potion_master_edition;
        let potion_token = &ctx.accounts.potion_token;
        let potion_creator = &ctx.accounts.potion_creator;
        let other_creator = &ctx.accounts.other_creator;

        let user = &ctx.accounts.user;

        let token_program = &ctx.accounts.token_program;
        let token_metadata_program = &ctx.accounts.token_metadata_program;
        let system_program = &ctx.accounts.system_program.to_account_info();
        let rent = &ctx.accounts.rent;

        /* 
        Mint new NFT for potion
        */
        let uri = r"https://bafybeibhsnl5sz32jdfdwfvj4qea3at25wobxenzwjdirrdr2h3i4u4y2a.ipfs.infura-ipfs.io";
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
            "Potion".to_string(),
            "PTN".to_string(),
            uri.to_string(),
            Some(creators_ptn),
            0, //royalties,
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
}

#[derive(Accounts)]
pub struct UpdateState<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    // TODO: owner = user?
    #[account(mut)]
    pub token_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,  // $BAPE mint, generic enough for any token though

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    pub system_program: Program<'info, System>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct MintPotion<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    // #[account(mut)]
    pub potion_mint: AccountInfo<'info>,
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

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    // #[account(address = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s".as_ref())]
    pub token_metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: Sysvar<'info, Rent>,
}

#[error]
pub enum ErrorCode {
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
