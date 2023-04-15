
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rusty_audio::Audio;
use std::{
    error::Error,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
    {io, thread},
};

use invaders::{
    frame::{self, new_frame, Drawable, Frame},
    invaders::Invaders,
    level::Level,
    menu::Menu,
    player::Player,
    render,
    score::Score,
};


fn main() -> Result <(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "audio/contributions/startupDoMiReDo/explode.wav");
    audio.add("lose", "audio/contributions/startupDoMiReDo/lose.wav");
    audio.add("move", "audio/contributions/startupDoMiReDo/move.wav");
    audio.add("pew", "audio/contributions/startupDoMiReDo/pew.wav");
    audio.add("startup", "audio/contributions/startupDoMiReDo/startup.wav");
    audio.add("win", "audio/contributions/startupDoMiReDo/win.wav");
    audio.play("startup");

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render Loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_grame, true);
        loop {
            match curr_frame = render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game Loop
    let mut player = mut Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();
    'gameloop: loop {
        //Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();
        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(" ") | KeyCode::Enter => {
                        if player.shot() {
                           audio.play("pew"); 
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop
                    }
                   _ => {} 
                }
            }
        }
    }

    // Updates
    player.update(delta);
    if invaders.update(delta) {
        audio.play("move")
    }


    // Draw & render
    let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
    for drawable in drawables {
        drawable.draw(&mut curr_frame);
    }
    let _ = render_tx.send(curr_frame);
    thread::sleep(Duration::from_millis(1));

    // Cleanup
    // drop(render_tx); -> for old version of Rust
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
