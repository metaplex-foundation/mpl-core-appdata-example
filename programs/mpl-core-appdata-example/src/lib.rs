use anchor_lang::prelude::*;

use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_external_plugin_adapter_data_info, fetch_plugin,
    instructions::{
        CreateCollectionV2CpiBuilder, CreateV2CpiBuilder, UpdatePluginV1CpiBuilder,
        WriteExternalPluginAdapterDataV1CpiBuilder,
    },
    types::{
        AppDataInitInfo, Attribute, Attributes, ExternalPluginAdapterInitInfo,
        ExternalPluginAdapterKey, ExternalPluginAdapterSchema, PermanentBurnDelegate,
        PermanentFreezeDelegate, PermanentTransferDelegate, Plugin, PluginAuthority,
        PluginAuthorityPair, PluginType, UpdateAuthority,
    },
    ID as MPL_CORE_ID,
};

declare_id!("BicWwtfJJAzWfAp2hzpdnyvvjB5TKnikAXxLVZbHcM2U");

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateEventArgs {
    pub name: String,
    pub uri: String,
    pub city: String,
    pub venue: String,
    pub artist: String,
    pub date: String,
    pub time: String,
    pub capacity: u64,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateTicketArgs {
    pub name: String,
    pub uri: String,
    pub hall: String,
    pub section: String,
    pub row: String,
    pub seat: String,
    pub price: u64,
    pub venue_authority: Pubkey,
}

#[program]
pub mod mpl_core_appdata_example {
    use super::*;

    pub fn setup_manager(ctx: Context<SetupManager>) -> Result<()> {
        ctx.accounts.manager.bump = ctx.bumps.manager;

        Ok(())
    }

    pub fn create_event(ctx: Context<CreateEvent>, args: CreateEventArgs) -> Result<()> {
        // Add an Attribute Plugin that will hold the event details
        let mut collection_plugin: Vec<PluginAuthorityPair> = vec![];

        let attribute_list: Vec<Attribute> = vec![
            Attribute {
                key: "City".to_string(),
                value: args.city,
            },
            Attribute {
                key: "Venue".to_string(),
                value: args.venue,
            },
            Attribute {
                key: "Artist".to_string(),
                value: args.artist,
            },
            Attribute {
                key: "Date".to_string(),
                value: args.date,
            },
            Attribute {
                key: "Time".to_string(),
                value: args.time,
            },
            Attribute {
                key: "Capacity".to_string(),
                value: args.capacity.to_string(),
            },
        ];
        collection_plugin.push(PluginAuthorityPair {
            plugin: Plugin::Attributes(Attributes { attribute_list }),
            authority: Some(PluginAuthority::UpdateAuthority),
        });

        // Create the Collection that will hold the tickets
        CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .collection(&ctx.accounts.event.to_account_info())
            .update_authority(Some(&ctx.accounts.manager.to_account_info()))
            .payer(&ctx.accounts.payer.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .name(args.name)
            .uri(args.uri)
            .plugins(collection_plugin)
            .invoke()?;

        Ok(())
    }

    pub fn create_ticket(ctx: Context<CreateTicket>, args: CreateTicketArgs) -> Result<()> {
        // Check that the maximum number of tickets has not been reached yet
        let (_, collection_attribute_list, _) = fetch_plugin::<BaseCollectionV1, Attributes>(
            &ctx.accounts.event.to_account_info(),
            PluginType::Attributes,
        )?;

        // Search for the Capacity attribute
        let capacity_attribute = collection_attribute_list
            .attribute_list
            .iter()
            .find(|attr| attr.key == "Capacity")
            .ok_or(TicketError::MissingAttribute)?;

        // Unwrap the Capacity attribute value
        let capacity = capacity_attribute
            .value
            .parse::<u32>()
            .map_err(|_| TicketError::NumericalOverflow)?;

        require!(
            ctx.accounts.event.num_minted < capacity,
            TicketError::MaximumTicketsReached
        );

        // Add an Attribute Plugin that will hold the ticket details
        let mut ticket_plugin: Vec<PluginAuthorityPair> = vec![];

        let attribute_list: Vec<Attribute> = vec![
            Attribute {
                key: "Ticket Number".to_string(),
                value: ctx
                    .accounts
                    .event
                    .num_minted
                    .checked_add(1)
                    .ok_or(TicketError::NumericalOverflow)?
                    .to_string(),
            },
            Attribute {
                key: "Hall".to_string(),
                value: args.hall,
            },
            Attribute {
                key: "Section".to_string(),
                value: args.section,
            },
            Attribute {
                key: "Row".to_string(),
                value: args.row,
            },
            Attribute {
                key: "Seat".to_string(),
                value: args.seat,
            },
            Attribute {
                key: "Price".to_string(),
                value: args.price.to_string(),
            },
        ];
        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::Attributes(Attributes { attribute_list }),
            authority: Some(PluginAuthority::UpdateAuthority),
        });
        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
            authority: Some(PluginAuthority::UpdateAuthority),
        });
        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            authority: Some(PluginAuthority::UpdateAuthority),
        });
        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            authority: Some(PluginAuthority::UpdateAuthority),
        });

        let ticket_external_plugin: Vec<ExternalPluginAdapterInitInfo> =
            vec![ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                data_authority: PluginAuthority::Address {
                    address: args.venue_authority,
                },
                schema: Some(ExternalPluginAdapterSchema::Binary),
            })];

        let signer_seeds = &[b"manager".as_ref(), &[ctx.accounts.manager.bump]];

        // Create the Ticket
        CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .asset(&ctx.accounts.ticket.to_account_info())
            .collection(Some(&ctx.accounts.event.to_account_info()))
            .payer(&ctx.accounts.payer.to_account_info())
            .authority(Some(&ctx.accounts.manager.to_account_info()))
            .owner(Some(&ctx.accounts.signer.to_account_info()))
            .system_program(&ctx.accounts.system_program.to_account_info())
            .name(args.name)
            .uri(args.uri)
            .plugins(ticket_plugin)
            .external_plugin_adapters(ticket_external_plugin)
            .invoke_signed(&[signer_seeds])?;

        Ok(())
    }

    pub fn scan_ticket(ctx: Context<ScanTicket>) -> Result<()> {
        let (_, app_data_length) = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
            &ctx.accounts.ticket.to_account_info(),
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::Address {
                address: ctx.accounts.signer.key(),
            }),
        )?;
        require!(app_data_length == 0, TicketError::AlreadyScanned);

        let data: Vec<u8> = "Scanned".as_bytes().to_vec();

        WriteExternalPluginAdapterDataV1CpiBuilder::new(
            &ctx.accounts.mpl_core_program.to_account_info(),
        )
        .asset(&ctx.accounts.ticket.to_account_info())
        .collection(Some(&ctx.accounts.event.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::Address {
                address: ctx.accounts.signer.key(),
            },
        ))
        .data(data)
        .invoke()?;

        let signer_seeds = &[b"manager".as_ref(), &[ctx.accounts.manager.bump]];

        UpdatePluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .asset(&ctx.accounts.ticket.to_account_info())
            .collection(Some(&ctx.accounts.event.to_account_info()))
            .payer(&ctx.accounts.payer.to_account_info())
            .authority(Some(&ctx.accounts.manager.to_account_info()))
            .system_program(&ctx.accounts.system_program.to_account_info())
            .plugin(Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate {
                frozen: true,
            }))
            .invoke_signed(&[signer_seeds])?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetupManager<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = Manager::INIT_SPACE,
        seeds = [MANAGER_SEEDS.as_bytes()],
        bump,
    )]
    pub manager: Account<'info, Manager>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateEvent<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = [MANAGER_SEEDS.as_bytes()],
        bump = manager.bump
    )]
    pub manager: Account<'info, Manager>,
    #[account(mut)]
    pub event: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: This is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CreateTicket<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = [MANAGER_SEEDS.as_bytes()],
        bump = manager.bump
    )]
    pub manager: Account<'info, Manager>,
    #[account(
        mut,
        constraint = event.update_authority == manager.key(),
    )]
    pub event: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub ticket: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: This is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct ScanTicket<'info> {
    pub owner: Signer<'info>,
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = [MANAGER_SEEDS.as_bytes()],
        bump = manager.bump
    )]
    pub manager: Account<'info, Manager>,
    #[account(
        mut,
        constraint = ticket.owner == owner.key(),
        constraint = ticket.update_authority == UpdateAuthority::Collection(event.key()),
    )]
    pub ticket: Account<'info, BaseAssetV1>,
    #[account(
        mut,
        constraint = event.update_authority == manager.key(),
    )]
    pub event: Account<'info, BaseCollectionV1>,
    pub system_program: Program<'info, System>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: This is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

const MANAGER_SEEDS: &str = "manager";

#[account]
pub struct Manager {
    pub bump: u8,
}

impl Space for Manager {
    const INIT_SPACE: usize = 8 + 1;
}

#[error_code]
pub enum TicketError {
    #[msg("The attribute is missing")]
    MissingAttribute,
    #[msg("Numerical Overflow")]
    NumericalOverflow,
    #[msg("The maximum number of tickets has been reached")]
    MaximumTicketsReached,
    #[msg("The ticket has already been scanned")]
    AlreadyScanned,
}
