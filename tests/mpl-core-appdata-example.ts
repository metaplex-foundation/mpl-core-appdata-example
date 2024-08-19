import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { MplCoreAppdataExample } from "../target/types/mpl_core_appdata_example";
import { BN } from "bn.js";

describe("mpl-core-appdata-example", () => {
  /// Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const wallet = anchor.Wallet.local();
  const program = anchor.workspace.MplCoreAppdataExample as Program<MplCoreAppdataExample>;

  const coreProgram = new PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d")

  const manager = PublicKey.createProgramAddressSync([Buffer.from("manager")], program.programId)[0];

  it("Setup Manager", async () => {
    const tx = await program.methods.setupManager()
    .accountsPartial({
      signer: wallet.publicKey,
      payer: wallet.publicKey,
      manager,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([wallet.payer])
    .rpc();

    console.log(tx);
  });

  const createEventArgs = {
    name: "Event 1",
    uri: "https://example.com",
    city: "City",
    venue: "Venue",
    artist: "Artist",
    date: "2022-01-01",
    time: "12:00",
    capacity: new BN(1000),
  }

  const event = Keypair.generate();

  it("Create Event", async () => {
    const tx = await program.methods.createEvent(createEventArgs)
    .accountsPartial({
      signer: wallet.publicKey,
      payer: wallet.publicKey,
      manager,
      event: event.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      mplCoreProgram: coreProgram,
    })
    .signers([wallet.payer, event])
    .rpc();

    console.log(tx);
  });

  const createTicketArgs = {
    name: "Ticket 1",
    uri: "https://example.com",
    hall: "Hall",
    section: "Section",
    row: "Row",
    seat: "Seat",
    price: new BN(1000),
    venueAuthority: wallet.publicKey,
  }

  const ticket = Keypair.generate();

  it("Create Ticket", async () => {
    const tx = await program.methods.createTicket(createTicketArgs)
    .accountsPartial({
      signer: wallet.publicKey,
      payer: wallet.publicKey,
      manager,
      event: event.publicKey,
      ticket: ticket.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      mplCoreProgram: coreProgram,
    })
    .signers([wallet.payer, ticket])
    .rpc();

    console.log(tx);
  });

  it("Scan Ticket", async () => {
    const tx = await program.methods.scanTicket()
    .accountsPartial({
      owner: wallet.publicKey,
      signer: wallet.publicKey,
      payer: wallet.publicKey,
      manager,
      ticket: ticket.publicKey,
      event: event.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      mplCoreProgram: coreProgram,
    })
    .signers([wallet.payer])
    .rpc();

    console.log(tx);
  });
});
