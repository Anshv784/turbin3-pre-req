<div align="center">

# pre-req-vault

**My submission for the builders prerequisite challenge.**

A small Anchor program that lets a user park SOL in a vault they control, and take it back
out again. For Task 2 I extended `withdraw` so that it also registers me with the cohort's
registration program through a CPI.

[**▶ Video walkthrough**](https://youtu.be/iKipa9F9PIY) &nbsp;·&nbsp; [**◈ Architecture diagram**](https://drive.google.com/file/d/1IldulslLU5i5u5VasRzyW0Byhk4_ahsN/view?usp=sharing)

`Anchor` &nbsp;·&nbsp; `Rust` &nbsp;·&nbsp; `Solana devnet`

</div>

---

## Deployed

| What | Address |
|---|---|
| My vault program | [`BAeWhhirV5TiQp3wwnatb9v5RTJLQBCETgFxj3Ueypxe`](https://explorer.solana.com/address/BAeWhhirV5TiQp3wwnatb9v5RTJLQBCETgFxj3Ueypxe?cluster=devnet) |
| Registration program | [`TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM`](https://explorer.solana.com/address/TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM?cluster=devnet) |
| My wallet | [`Cbdvwy6Dm7tbCsLP3nw4Umz29BLNQkNwCBDDDRrkbpTZ`](https://explorer.solana.com/address/Cbdvwy6Dm7tbCsLP3nw4Umz29BLNQkNwCBDDDRrkbpTZ?cluster=devnet) |
| Application PDA | [`Kck8C7RQ83u8MSPxkiGonBSFsFr4xTR1VHQPD41s18B`](https://explorer.solana.com/address/Kck8C7RQ83u8MSPxkiGonBSFsFr4xTR1VHQPD41s18B?cluster=devnet) — github `Anshv784` |

All on **devnet**.

## How the vault works

The program itself holds no data. Everything lives in two PDAs derived from the user's key,
so every user gets their own vault and nobody can touch anybody else's.

**`VaultState`** — seeds `["state", user]`. A tiny account that only stores the two bumps,
so later instructions don't have to search for them again.

**`vault`** — seeds `["vault", vault_state]`. A plain `SystemAccount` with no data on it. It
just holds lamports. Because it's a PDA it sits off the ed25519 curve, so no private key for
it exists anywhere and the only way funds move out is the program signing with the seeds.

The four instructions:

| Instruction | What happens | Who signs the transfer |
|---|---|---|
| `initialize` | Creates `VaultState`, saves `vault_bump` and `state_bump` | user pays rent |
| `deposit(amount)` | Moves `amount` from the user into the vault | the user |
| `withdraw(amount)` | Moves `amount` from the vault back to the user | the vault PDA, via seeds |
| `close` | Sends everything left back and deletes `VaultState` | the vault PDA, via seeds |

Every lamport movement is a CPI into the System Program. Deposit is signed by the user
normally. Withdraw and close can't be, since nobody holds the vault's key, so they use
`CpiContext::new_with_signer` with `["vault", vault_state, bump]` and the runtime accepts the
program's signature for that address.

## What I changed for Task 2

`withdraw` was already given the two accounts it needed (`application_account` and
`application_program`) but wasn't using them. I added the CPI in
[`instructions/withdraw.rs`](programs/pre-req-vault/src/instructions/withdraw.rs), right
after the SOL transfer:

```rust
let cpi_accounts = Initialize {
    user: self.user.to_account_info(),
    account: self.application_account.to_account_info(),
    system_program: self.system_program.to_account_info(),
};

let cpi_ctx = CpiContext::new(self.application_program.key(), cpi_accounts);

initialize(cpi_ctx, GITHUB_USERNAME.to_string())?;
```

The registration program wants `user` as a `Signer`. That's free here: the user already
signed the outer transaction, and a signature carries down into a CPI. So unlike the transfer
above it, this call needs no seeds.

The GitHub username is a constant in `constants.rs`, which keeps `withdraw`'s signature and
the existing tests unchanged.

I deployed my own copy rather than calling the one that was already up. `anchor keys sync`
gave me a fresh program ID, which is in `declare_id!` and `Anchor.toml`.

## Proof it worked

Withdraw transaction:
[`2QWscEH1B2MV3myySR5QLdwKDDztqnuhUvX7bS7owsua4DSDc34pmJHvLY7PQcbenyqm93tD6KANMnzfyWZ1qwp7`](https://explorer.solana.com/tx/2QWscEH1B2MV3myySR5QLdwKDDztqnuhUvX7bS7owsua4DSDc34pmJHvLY7PQcbenyqm93tD6KANMnzfyWZ1qwp7?cluster=devnet)

```
Program BAeWhhirV5TiQp3wwnatb9v5RTJLQBCETgFxj3Ueypxe invoke [1]
  Program 11111111111111111111111111111111 invoke [2]      <- the SOL back to me
  Program TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM invoke [2]   <- the registration CPI
    Program 11111111111111111111111111111111 invoke [3]
    Program 11111111111111111111111111111111 success
  Program TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM success
Program BAeWhhirV5TiQp3wwnatb9v5RTJLQBCETgFxj3Ueypxe success
```

`[1]` is my program and `[2]` is the registration program, so it's a real CPI and not a
direct call.

The application account now reads:

```
owner  : TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM
github : "Anshv784"
```

## Running it

```bash
pnpm install
anchor build
anchor deploy
anchor test --skip-build --skip-deploy
```

Anchor.toml points at devnet and at my keypair, so change `[provider].wallet` to yours.

Note that the whole suite only passes once per wallet. Registration is one-shot, so a second
run fails on `withdraw` with "account already in use" — that's the registration program doing
its job, not a bug in the vault.

If `anchor test` dies on `pnpm install` before it reaches the tests, that's pnpm 11 refusing to
finish while the `bufferutil` / `utf-8-validate` build scripts are unanswered. Either run
`pnpm approve-builds`, or drop a `pnpm-workspace.yaml` next to `package.json` with:

```yaml
allowBuilds:
  bufferutil: false
  utf-8-validate: false
```
