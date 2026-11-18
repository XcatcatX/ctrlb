use super::commands::{self, Next};
use super::{console_control, input, messages, signals};
use crate::runtime::Runtime;
use crate::tray::{Tray, TrayCommand};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

type ExitSignal = Pin<Box<dyn Future<Output = ()>>>;
type Lines = UnboundedReceiver<String>;

#[derive(PartialEq)]
enum Step {
    Quit,
    Tray,
}

enum TrayEnd {
    Quit,
    BackToTerminal,
    Failed,
}

enum Leave {
    Quit,
    Terminal,
}

/// Runs the terminal and the tray. Ends when the program should close.
pub async fn run_session(runtime: &mut Runtime) {
    let mut exit_signal: ExitSignal = Box::pin(signals::wait_for_exit_signal());
    let mut lines = input::start_input_thread();
    loop {
        if run_terminal(runtime, &mut lines, &mut exit_signal).await == Step::Quit {
            return;
        }
        match run_tray(runtime, &mut exit_signal).await {
            TrayEnd::Quit => return,
            // A new terminal window needs a new input thread.
            TrayEnd::BackToTerminal => lines = input::start_input_thread(),
            TrayEnd::Failed => {}
        }
    }
}

/// Runs commands from the terminal until "stop", "tray" or Ctrl+C.
async fn run_terminal(
    runtime: &mut Runtime,
    lines: &mut Lines,
    exit_signal: &mut ExitSignal,
) -> Step {
    loop {
        tokio::select! {
            line = lines.recv() => match line {
                Some(text) => match commands::run_command(&text, runtime).await {
                    Next::Continue => {}
                    Next::Tray => return Step::Tray,
                    Next::Quit => return Step::Quit,
                },
                None => {
                    // The terminal input is closed. Keep running until the program is closed.
                    exit_signal.as_mut().await;
                    return stop_everything(runtime).await;
                }
            },
            _ = exit_signal.as_mut() => return stop_everything(runtime).await,
        }
    }
}

async fn stop_everything(runtime: &mut Runtime) -> Step {
    println!("Stopping...");
    runtime.stop().await;
    Step::Quit
}

/// Closes the terminal and keeps the program running in the tray.
async fn run_tray(runtime: &mut Runtime, exit_signal: &mut ExitSignal) -> TrayEnd {
    let mut tray = match Tray::start() {
        Ok(tray) => tray,
        Err(message) => {
            println!("{message}");
            return TrayEnd::Failed;
        }
    };
    println!("The terminal closes now. The program keeps running in the tray.");
    tokio::time::sleep(Duration::from_millis(800)).await;
    console_control::close_console();

    let leave = wait_in_tray(runtime, &mut tray, exit_signal).await;
    tray.stop().await;
    match leave {
        Leave::Quit => {
            runtime.stop().await;
            TrayEnd::Quit
        }
        Leave::Terminal => {
            console_control::open_console();
            messages::print_welcome_back(runtime.state());
            TrayEnd::BackToTerminal
        }
    }
}

async fn wait_in_tray(
    runtime: &Runtime,
    tray: &mut Tray,
    exit_signal: &mut ExitSignal,
) -> Leave {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            command = tray.next_command() => return match command {
                Some(TrayCommand::Quit) => Leave::Quit,
                Some(TrayCommand::ShowTerminal) | None => Leave::Terminal,
            },
            _ = ticker.tick() => tray.show_link(runtime.state().tunnel_link()),
            _ = exit_signal.as_mut() => return Leave::Quit,
        }
    }
}
