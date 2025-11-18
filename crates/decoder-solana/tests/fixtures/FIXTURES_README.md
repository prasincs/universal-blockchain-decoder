# Solana Transaction Test Fixtures

This directory contains test fixtures for the Solana decoder. Each fixture consists of:
- A `.base64` file containing the base64-encoded transaction binary
- A `.json` file containing the expected decoded structure

## Fixtures

### 1. simple_system_transfer
**Location**: `simple/simple_system_transfer.{base64,json}`

A minimal Solana transaction performing a simple SOL transfer.

- **Type**: Legacy Transaction
- **Instructions**: 1 (System transfer)
- **Signers**: 1 (payer)
- **Accounts**: 3 (Alice, Bob, System Program)
- **Description**: Transfers 5,000,000 lamports from Alice to Bob
- **Key Features**:
  - Simple account-based transfer
  - Single instruction
  - No readonly accounts
  - No complex logic

**Binary Size**: 215 bytes

### 2. multi_instruction_transfer
**Location**: `simple/multi_instruction_transfer.{base64,json}`

A transaction with multiple sequential instructions.

- **Type**: Legacy Transaction
- **Instructions**: 3 (Allocate, Assign, Transfer)
- **Signers**: 2 (Alice as payer, NewAccount as authority)
- **Accounts**: 4 (Alice, NewAccount, Bob, System Program)
- **Description**: 
  1. Allocates space for a new account (1,000,000,000 bytes)
  2. Assigns the new account to a program
  3. Transfers 5,000,000 lamports to Bob
- **Key Features**:
  - Multiple signers required
  - Multiple related instructions
  - Account creation flow
  - Demonstrates instruction ordering

**Binary Size**: 336 bytes

### 3. transfer_with_readonly_account
**Location**: `simple/transfer_with_readonly_account.{base64,json}`

A transaction that includes readonly accounts.

- **Type**: Legacy Transaction
- **Instructions**: 1 (System transfer)
- **Signers**: 1 (Alice)
- **Accounts**: 4 (Alice, Bob, Observer, System Program)
- **Description**: Transfers 2,500,000 lamports with a readonly observer account
- **Key Features**:
  - Readonly unsigned account
  - Message header distinguishes account types
  - Demonstrates account role segregation

**Binary Size**: 247 bytes

### 4. transfer_with_nonce
**Location**: `simple/transfer_with_nonce.{base64,json}`

A transaction using durable nonce for replay protection.

- **Type**: Legacy Transaction
- **Instructions**: 2 (Advance nonce, Transfer)
- **Signers**: 2 (Alice as payer, NonceAuthority)
- **Accounts**: 6 (Alice, NonceAccount, NonceAuthority, RecentBlockhashes sysvar, Bob, System Program)
- **Description**: 
  1. Advances the durable nonce (must be first instruction)
  2. Transfers 3,000,000 lamports to Bob
- **Key Features**:
  - Durable nonce mechanism
  - Sysvar account reference (RecentBlockhashes)
  - Readonly unsigned accounts
  - Prevents replay attacks

**Binary Size**: 385 bytes

### 5. multi_signer_transaction
**Location**: `simple/multi_signer_transaction.{base64,json}`

A transaction requiring 3 signatures.

- **Type**: Legacy Transaction
- **Instructions**: 2 (Custom instruction, Transfer)
- **Signers**: 3 (Accounts 0, 1, 2)
- **Accounts**: 5 (Signer1, Signer2, Signer3, Account4, Account5)
- **Description**: 
  1. Custom instruction involving all 3 signers
  2. Transfer of 1,000,000 lamports
- **Key Features**:
  - Multiple required signers
  - All signers must sign the message
  - Demonstrates multisig scenarios

**Binary Size**: 418 bytes

## Transaction Format Reference

Each fixture follows Solana's legacy transaction format:

```
Transaction {
  signatures: Vec<Signature>         // Compact-encoded, 64 bytes each
  message: Message {
    header: MessageHeader {
      num_required_signatures: u8
      num_readonly_signed_accounts: u8
      num_readonly_unsigned_accounts: u8
    }
    account_keys: Vec<Pubkey>        // 32 bytes each, compact-encoded count
    recent_blockhash: Hash           // 32 bytes
    instructions: Vec<Instruction> {
      program_id_index: u8
      accounts: Vec<u8>              // Compact-encoded indices
      data: Vec<u8>                  // Compact-encoded data
    }
  }
}
```

## System Program Instructions

The fixtures use the following System Program instruction types:

- **0x02** - Transfer: `struct { type: u32, lamports: u64 }`
- **0x03** - Assign: `struct { type: u32 }`
- **0x04** - AdvanceNonceAccount: `struct { type: u32 }`
- **0x08** - Allocate: `struct { type: u32, space: u64 }`

## Usage

To decode a fixture:

```bash
# Decode from base64
cat simple/simple_system_transfer.base64 | base64 -d > tx.bin

# Use decoder on tx.bin
solana-decoder decode --input tx.bin --format binary

# Compare with expected output
diff <(solana-decoder decode --format json) simple/simple_system_transfer.json
```

## Sources

These fixtures were generated from:
1. Solana SDK test code patterns (`/home/user/universal-blockchain-decoder/tmp/fixture_fetch/solana/sdk/src/transaction/`)
2. Manual construction following Solana transaction format specification
3. Binary encoding using compact-array serialization matching bincode format

## Notes

- Signatures in fixtures use dummy 64-byte values (all zeros) for testing binary structure
- Blockhashes are dummy 32-byte values (all zeros) for testing
- Pubkeys use patterned bytes (0x01-0x02 for simple_system_transfer, etc.) to distinguish accounts
- The transaction format used is Solana's **legacy message format** (not V0)
