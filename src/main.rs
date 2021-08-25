use hidapi;
use std::thread;
use std::time::Duration;

use log::{debug, error, info};
use tfc::{Context, traits::*, Key};

#[derive(Copy, Clone)]
#[repr(u8)]
enum Buttons {
    Y = 0x01,
    B = 0x02,
    A = 0x04,
    X = 0x08,
    L = 0x10,
    R = 0x20,
    ZL = 0x40,
    ZR = 0x80,
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum Extra {
    Minus = 0x01,
    Plus = 0x02,
    LSB = 0x04,
    RSB = 0x08,
    Home = 0x10,
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum Dpad {
    Off = 0x00,
    U = 0x01,
    R = 0x02,
    D = 0x04,
    L = 0x08,
    UR = 0x03,
    DR = 0x06,
    UL = 0x09,
    DL = 0x0C,
}

#[derive(Debug)]
struct Input {
    buttons: u8,
    extra: u8,
    dpad: u8,
    lstick1: u8,
    lstick2: u8,
    rstick1: u8,
    rstick2: u8,
    unused: u8,
}

impl Input {
    fn new(data: [u8; 8]) -> Input {
        Input {
            buttons: data[0],
            extra: data[1],
            dpad: data[2],
            lstick1: data[3],
            lstick2: data[4],
            rstick1: data[5],
            rstick2: data[6],
            unused: data[7],
        }
    }

    fn default() -> Input {
        Input {
            buttons: 0,
            extra: 0,
            dpad: 15,
            lstick1: 128,
            lstick2: 128,
            rstick1: 128,
            rstick2: 128,
            unused: 0,
        }
    }
}

#[derive(Debug)]
struct State {
    buttons: u8,
    extra: u8,
    dpad: u8,
}

impl State {
    fn new() -> State {
        State {
            buttons: 0,
            extra: 0,
            dpad: 0,
        }
    }
}

#[derive(Debug)]
struct Controller {
    state: State,
}

impl Controller {
    fn new() -> Controller {
        Controller {
            state: State::new()
        }
    }

    fn clear_state(&mut self) {
        self.state.buttons = 255;
        self.state.extra = 255;
        self.state.dpad = 255;
    }

    fn update(&mut self, input: Input, ctx: &mut Context) -> Result<(), tfc::Error> {
        self.state.buttons = self._handle_buttons(input.buttons, ctx)?;
        self.state.extra = self._handle_extra(input.extra, ctx)?;
        self.state.dpad = self._handle_dpad(input.dpad, ctx)?;

        Ok(())
    }

    fn _handle_buttons(&self, buttons: u8, ctx: &mut Context) -> Result<u8, tfc::Error> {
        let mut s: u8 = 0;
        let diff = buttons ^ self.state.buttons;

        s |= self._check_key(buttons, diff, Buttons::Y as u8, Key::P, ctx)?;
        s |= self._check_key(buttons, diff, Buttons::B as u8, Key::O, ctx)?;
        s |= self._check_key(buttons, diff, Buttons::A as u8, Key::I, ctx)?;
        s |= self._check_key(buttons, diff, Buttons::X as u8, Key::U, ctx)?;
        s |= self._check_key(buttons, diff, Buttons::L as u8, Key::Y, ctx)?;
        s |= self._check_key(buttons, diff, Buttons::R as u8, Key::T, ctx)?;
        s |= self._check_key(buttons, diff, Buttons::ZL as u8, Key::R, ctx)?;
        s |= self._check_key(buttons, diff, Buttons::ZR as u8, Key::E, ctx)?;

        Ok(s)
    }

    fn _handle_extra(&self, extra: u8, ctx: &mut Context) -> Result<u8, tfc::Error> {
        let mut s: u8 = 0;
        let diff = extra ^ self.state.extra;

        s |= self._check_key(extra, diff, Extra::Minus as u8, Key::L, ctx)?;
        s |= self._check_key(extra, diff, Extra::Plus as u8, Key::K, ctx)?;
        s |= self._check_key(extra, diff, Extra::LSB as u8, Key::J, ctx)?;
        s |= self._check_key(extra, diff, Extra::RSB as u8, Key::H, ctx)?;
        s |= self._check_key(extra, diff, Extra::Home as u8, Key::G, ctx)?;

        Ok(s)
    }

    fn _handle_dpad(&self, dpad: u8, ctx: &mut Context) -> Result<u8, tfc::Error> {
        let mut s: u8 = 0;
        let cleaned = self._convert_dpad(dpad) as u8;
        let diff = cleaned ^ self.state.dpad;

        s |= self._check_key(cleaned, diff, Dpad::U as u8, Key::W, ctx)?;
        s |= self._check_key(cleaned, diff, Dpad::D as u8, Key::S, ctx)?;
        s |= self._check_key(cleaned, diff, Dpad::L as u8, Key::A, ctx)?;
        s |= self._check_key(cleaned, diff, Dpad::R as u8, Key::D, ctx)?;

        Ok(s)
    }

    fn _check_key(&self, input: u8, diff: u8, button: u8, key: Key, ctx: &mut Context) -> Result<u8, tfc::Error> {
        if diff & button != 0 {
            let d = input & button;

            if d != 0 { ctx.key_down(key)?; }
            else { ctx.key_up(key)?; }

            return Ok(d);
        }

        Ok(0)
    }

    fn _convert_dpad(&self, dpad: u8) -> Dpad {
        match dpad {
            0 => Dpad::U,
            1 => Dpad::UR,
            2 => Dpad::R,
            3 => Dpad::DR,
            4 => Dpad::D,
            5 => Dpad::DL,
            6 => Dpad::L,
            7 => Dpad::UL,
            _ => Dpad::Off,
        }
    }
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

    let target = "HORIPAD S";
    let mut ctx = Context::new().unwrap();
    thread::sleep(Duration::from_millis(10));

    match hidapi::HidApi::new() {
        Ok(api) => {
            match open_target(&api, target) {
                Some(device) => { poll(&device, &mut ctx) },
                None => {}
            };
        }
        Err(e) => error!("Error connecting device {:?}", e)
    }

    info!("Shutting down...")
}

fn poll(device: &hidapi::HidDevice, ctx: &mut Context) {
    info!("Polling Device...");
    let mut controller = Controller::new();
    let mut i: u8 = 0;

    loop {
        // need to clear every so often to handle dropped inputs
        if i % 7 == 0 { controller.clear_state() }
        i = i.wrapping_add(1);

        match read_input(device) {
            Ok(input) => { 
                match controller.update(input, ctx) {
                    Ok(_) => continue,
                    Err(_) => controller.clear_state()
                }
            },
            Err(_) => { 
                match controller.update(Input::default(), ctx) { // assume no input
                    Ok(_) => continue,
                    Err(_) => controller.clear_state()
                }
            }
        }
    }
}

fn read_input(device: &hidapi::HidDevice) -> Result<Input, hidapi::HidError> {
    // Read data from device
    let mut buf = [0u8; 8];

    device.read(&mut buf[..])?;
    let input = Input::new(buf);
    debug!("Read: {:?}", &input);

    thread::sleep(Duration::from_millis(1));
    Ok(input)
}

fn open_target(api: &hidapi::HidApi, target: &str) -> Option<hidapi::HidDevice> {
    for device_info in api.devices() {
        if device_info.product_string.as_ref()? == target {
            info!("Opening device...");

            return match device_info.open_device(api) {
                Ok(device) => {
                    let manufacturer = device.get_manufacturer_string().unwrap_or_default().unwrap_or_default();
                    let product = device.get_product_string().unwrap_or_default().unwrap_or_default();
                    info!("Product: {:?}, manufacturer: {:?}", product, manufacturer);
                    Some(device)
                },
                Err(e) => {
                    error!("Could not open device: {:?}", e);
                    None
                }
            };
        }
    }

    error!("Unable to find provided target {:?}", target);
    None
}
