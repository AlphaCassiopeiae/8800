mod cpu;
use cpu::*;
mod ram;
use ram::*;
mod instructions;
use instructions::*;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {

    let mut ram: RAM = RAM::new();
    ram.init();
    let mut cpu: CPU = CPU::new(ram);
    cpu.reset();

    enable_raw_mode().expect("Failed to enable raw mode");

    loop {

        let has_event = tokio::task::spawn_blocking(|| {
            event::poll(Duration::from_millis(100))
        })
        .await
        .unwrap()
        .unwrap();

        if has_event {
            let ev = tokio::task::spawn_blocking(event::read)
                .await
                .unwrap()
                .unwrap();

            if let Event::Key(key_event) = ev {
                match key_event.code {
                    KeyCode::Char('p') => {
                        // print CPU state (regs, pc, sp, flags)
                        println!("{}", cpu);
                    }
                    KeyCode::Char('n') => {
                        // let instr: u8 = ram.read(cpu.read_pc().into());
                        let instr: u8 = cpu.fetch_instruction();

                        cpu.execute_instruction(instr);

                        if cpu.halted() {
                            println!("HLT detected, exiting...\r");
                            break;
                        }
                    }
                    KeyCode::Char('r') => {
                        loop {
                            let instr: u8 = cpu.fetch_instruction();

                            cpu.execute_instruction(instr);

                            if cpu.halted() {
                                println!("HLT detected, exiting...\r");
                                break;
                            }
                            time::sleep(Duration::from_millis(50)).await;
                        }
                    }
                    KeyCode::Char('q') => {
                        println!("Exiting...\r");
                        break;
                    }
                    KeyCode::Char(c) => println!("You pressed: {}\r", c),
                    _ => println!("Other key pressed\r"),
                }
            }
        } else {
            time::sleep(Duration::from_millis(50)).await;
        }
    }
    
    disable_raw_mode().expect("Failed to disable raw mode");
}