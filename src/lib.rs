pub mod state;
pub mod instructions;
pub mod processor;

use processor::process_instruction;
use solana_program::entrypoint;

entrypoint!(process_instruction);

