
use std::time::{Instant, Duration};
use glium::glutin::{EventsLoop, Event};

const NANOS_PER_SEC: u64 = 1_000_000_000;

pub struct SchedulerSettings {

    fps: u32,

    dt_f: u64,

    ups: u32,

    dt_u: u64,

    ups_reset: u32,

    lazy: bool,

}

impl Default for SchedulerSettings {

    fn default() -> Self {
        SchedulerSettings {
            fps: 60,
            dt_f: NANOS_PER_SEC / 60,
            ups: 60,
            dt_u: NANOS_PER_SEC / 60,
            ups_reset: 0,
            lazy: false
        }
    }
}

impl SchedulerSettings {

    pub fn set_fps(&mut self, fps: u32) -> &mut Self {
        self.fps = fps;
        if self.fps > 0 {
            self.dt_f = NANOS_PER_SEC / self.fps as u64;
        }
        self
    }

    pub fn set_ups(&mut self, ups: u32) -> &mut Self {
        self.ups = ups;
        if self.ups > 0 {
            self.dt_u = NANOS_PER_SEC / self.ups as u64;
        }
        self
    }

    pub fn set_ups_reset(&mut self, ups_reset: u32) -> &mut Self {
        self.ups_reset = ups_reset;
        self
    }

    pub fn set_lazy(&mut self, lazy: bool) -> &mut Self {
        self.lazy = lazy;
        self
    }

}



use glium::glutin::{WindowBuilder, ContextBuilder, ContextCurrentState};
use glium::{Display};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait GameLogic {

    fn init(&mut self, display: &Display, settings: &mut SchedulerSettings) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, dt: u64, display: &Display, settings: &mut SchedulerSettings) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, dt: u64, settings: &mut SchedulerSettings) -> Result<()> {
        Ok(())
    }

    fn handle_event(&mut self, event: Event, settings: &mut SchedulerSettings, close: &mut bool) -> Result<()> {
        match event {
            Event::WindowEvent{window_id, event} => match event {
                glium::glutin::WindowEvent::CloseRequested => { *close = true },
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn finalize(&mut self, err: Option<Box<dyn std::error::Error>>) -> Result<()> {
        Ok(())
    }
}

pub trait GameClock {

    // reset and return nanoseconds since last reset
    fn reset(&mut self) -> u64;

    // nanoseconds since reset
    fn elapsed(&mut self) -> u64;

}

enum State {
    Render,
    SwapBuffers,
    HandleEvents,
    Update,
    Sleep,
}

pub struct Scheduler<'a> {

    settings: SchedulerSettings,

    last_update: u64,

    lag_update: u64,

    last_render: u64,

    game_logic: Box<dyn GameLogic + 'a>,

    game_clock: Box<dyn GameClock + 'a>,

    eventsloop: EventsLoop,

    display: Display,
}

impl<'a> Default for Scheduler<'a> {

    fn default() -> Self {
        let eventsloop = EventsLoop::new();
        let display = Display::new(WindowBuilder::new(), ContextBuilder::new(), &eventsloop).unwrap();
        Scheduler {
            settings: SchedulerSettings::default(),
            last_update: 0,
            lag_update: 0,
            last_render: 0,          
            game_logic: Box::new(EmptyGameLogic::default()),
            game_clock: Box::new(StdGameClock::default()),
            eventsloop,
            display
        }
    }
}

impl<'a> Scheduler<'a> {

    pub fn new<T: ContextCurrentState, L: GameLogic + 'a, C: GameClock + 'a>(wb: WindowBuilder, cb: ContextBuilder<'a, T>, settings: SchedulerSettings, game_logic: L, game_clock: C) -> Self {
        let eventsloop = EventsLoop::new();
        let display = Display::new(wb, cb, &eventsloop).unwrap();
        Scheduler {
            settings,
            last_update: 0,
            lag_update: 0,
            last_render: 0,
            game_logic: Box::new(game_logic),
            game_clock: Box::new(game_clock),
            eventsloop,
            display
        }
    }

    #[deprecated()]
    pub fn set_game_logic<L: GameLogic + 'a>(mut self, game_logic: L) -> Self {
        self.game_logic = Box::new(game_logic);
        self
    }

    #[deprecated()]
    pub fn set_game_clock<C: GameClock + 'a>(mut self, game_clock: C) -> Self {
        self.game_clock = Box::new(game_clock);
        self
    }

    pub fn get_eventsloop(&self) -> &EventsLoop {
        &self.eventsloop
    }

    pub fn run(&mut self) -> Result<()> {
        let settings = &mut self.settings;
        let logic = self.game_logic.as_mut();
        let clock = self.game_clock.as_mut();
        let eventsloop = &mut self.eventsloop;
        clock.reset();
        let mut now = clock.elapsed();
        self.last_update = 0;
        let mut next_update = now;
        self.last_render = 0;
        let mut next_render = now;
        let mut res = None;
        let mut state = State::Update;
        if let Err(e) = logic.init(&self.display, settings) {
            res = Some(e);
        } else {
            loop {

                //render
                if let State::Update = state {
                    now = clock.elapsed();
                }
                if now >= next_render {
                    if let Err(e) = logic.render(now - self.last_render, &self.display, settings) {
                        res = Some(e); 
                        break;
                    }
                    self.last_render = now;
                    Self::update_time(&mut next_render, now, settings.dt_f, 1);
                    state = State::Render;
                }

                // sleep
                now = clock.elapsed();
                if now < next_render && now < next_update {
                    let wait = std::cmp::min(next_render - now, next_update - now);
                    state = State::Sleep;
                    std::thread::sleep(Duration::from_nanos(wait)); 
                }

                //input
                let mut close = false;
                eventsloop.poll_events(|evt| {
                    if let None = res {
                        if !close {
                            if let Err(e) = logic.handle_event(evt, settings, &mut close) {
                                res = Some(e);
                            } else {
                                state = State::HandleEvents;
                            }
                        }
                    }
                });
                if let Some(_) = res {
                    break;
                }
                if close {
                    break;
                }

                // update
                if let State::HandleEvents = state {
                    now = clock.elapsed();
                }
                if now >= next_update {
                    if let Err(e) = logic.update(now - self.last_update, settings) {
                        res = Some(e); 
                        break; 
                    }
                    self.last_update = now;
                    self.lag_update = Self::update_time(&mut next_update, now, settings.dt_u, settings.ups_reset as u64);
                    state = State::Update;
                }                                      
            }
        }
        logic.finalize(res)
    }

    fn update_time(next: &mut u64, now: u64, dt: u64, max_lag: u64) -> u64 {
        let mut lag = (now - *next) / dt;
        if max_lag > 0 && lag >= max_lag {
            *next += lag * dt;
            lag = 0;
        }
        *next += dt;
        lag
    }
}



pub struct StdGameClock {
    base: Instant,
}

impl GameClock for StdGameClock {

    fn reset(&mut self) -> u64 {
        let old = self.base;
        self.base = Instant::now();
        super::util::duration_to_nanos(self.base - old)
    }

    fn elapsed(&mut self) -> u64 {
        let now = Instant::now();
        super::util::duration_to_nanos(now - self.base)
    }
}

impl Default for StdGameClock {
    fn default() -> Self {
        StdGameClock {
            base: Instant::now()
        }
    }
}



pub struct EmptyGameLogic {
    
}

impl GameLogic for EmptyGameLogic {

    fn init(&mut self, display: &Display, settings: &mut SchedulerSettings) -> Result<()> {
        use glium::Surface;
        println!("EmptyGameLogic");
        let mut target = display.draw();
        target.clear_color(0.5, 0.5, 0.5, 0.1);
        target.finish().map_err(Box::new)?;
        Ok(())
    }
   
}

impl Default for EmptyGameLogic {

    fn default() -> Self {
        EmptyGameLogic {

        }
    }
}









