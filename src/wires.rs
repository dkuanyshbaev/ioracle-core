use rand::distributions::{Distribution, Uniform};
use rppal::gpio::Gpio;
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};
use serialport::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, SeekFrom};
use std::time::{Duration, SystemTime};
use std::{process, thread};

const PUMP_FILE: &str = "/tmp/pumps.usage";
const LEDS_IN_LINE: i32 = 144;

const DEFAULT_COLOUR: &str = "rgb(51, 0, 180)";
const LI_COLOUR: &str = "rgb(230, 4, 211)";
const HEAVEN_COLOUR: &str = "rgb(224, 4, 235)";
const CLOUD_COLOUR: &str = "rgb(255, 2, 14)";
const SUN_COLOUR: &str = "rgb(255, 109, 0)";
const WIND_COLOUR: &str = "rgb(121, 255, 0)";
const THUNDER_COLOUR: &str = "rgb(255, 53, 6)";
const WATER_COLOUR: &str = "rgb(38, 2, 255)";
const MOUNTAIN_COLOUR: &str = "rgb(14, 255, 232)";
const EARTH_COLOUR: &str = "rgb(0, 0, 0)";

// LEDs controller
pub fn build_controller(brightness: u8) -> Option<Controller> {
    match ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0,
            ChannelBuilder::new()
                .pin(12)
                .count(6 * LEDS_IN_LINE)
                .strip_type(StripType::Ws2811Rgb)
                .brightness(brightness)
                .build(),
        )
        .channel(
            1,
            ChannelBuilder::new()
                .pin(13)
                .count(3 * LEDS_IN_LINE)
                .strip_type(StripType::Ws2811Rgb)
                .brightness(brightness)
                .build(),
        )
        .build()
    {
        Ok(controller) => Some(controller),
        Err(_) => None,
    }
}

// render the line
pub fn render(l: u8, line_num: i32, controller: &mut Controller, colour: &String) {
    match l {
        1 => render_yang(line_num, controller, colour),
        _ => render_yin(line_num, controller, colour),
    }
}

pub fn render_yin(line_num: i32, controller: &mut Controller, colour: &String) {
    let leds = controller.leds_mut(0);
    let (a, b, c) = parse_colour(colour);

    let part = LEDS_IN_LINE / 3;
    let position = LEDS_IN_LINE * (line_num - 1);
    for num in position..position + LEDS_IN_LINE {
        if num > position + part && num < position + part * 2 {
            leds[num as usize] = [0, 0, 0, 0];
        } else {
            leds[num as usize] = [c, a, b, 0];
        }
    }

    if let Err(e) = controller.render() {
        println!("{:?}", e);
    };
}

pub fn render_yang(line_num: i32, controller: &mut Controller, colour: &String) {
    let leds = controller.leds_mut(0);
    let (a, b, c) = parse_colour(colour);

    let position = LEDS_IN_LINE * (line_num - 1);
    for num in position..position + LEDS_IN_LINE {
        leds[num as usize] = [c, a, b, 0];
    }

    if let Err(e) = controller.render() {
        println!("{:?}", e);
    };
}

// render resting state for the lines
pub fn render_resting(controller: &mut Controller) {
    let yao = controller.leds_mut(0);
    for num in 0..yao.len() {
        yao[num as usize] = [0, 0, 0, 0];
    }

    let (a, b, c) = parse_colour(&LI_COLOUR.to_string());
    let li = controller.leds_mut(1);
    for num in 0..li.len() {
        li[num as usize] = [c, b, a, 0];
    }

    if let Err(e) = controller.render() {
        println!("Resting error: {:?}", e);
    }
}

pub fn render_li(controller: &mut Controller) {
    let li = controller.leds_mut(1);
    let mut rng1 = rand::thread_rng();
    let mut rng2 = rand::thread_rng();
    let blue_range = Uniform::from(54..200);

    for num in 0..li.len() {
        let b = blue_range.sample(&mut rng1);
        let green_range = Uniform::from(0..b / 4);
        let g = green_range.sample(&mut rng2);
        li[num as usize] = [b, g, 0, 0];
    }

    if let Err(e) = controller.render() {
        println!("Li error: {:?}", e);
    }
}

pub fn drop_li_to_default(controller: &mut Controller) {
    let (a, b, c) = parse_colour(&LI_COLOUR.to_string());
    let li = controller.leds_mut(1);
    for num in 0..li.len() {
        li[num as usize] = [c, b, a, 0];
    }

    if let Err(e) = controller.render() {
        println!("Li error: {:?}", e);
    }
}

pub fn reading_no_led() -> (String, String) {
    println!("New reading... No leds");

    // let yao = controller.leds_mut(0);
    // for num in 0..LEDS_IN_LINE * 6 {
    //     yao[num as usize] = [0, 0, 0, 0];
    // }
    // if let Err(e) = controller.render() {
    //     println!("Yao error while reading: {:?}", e);
    // };

    //---------------------------------------------------
    let m = "1".to_string();
    let b = "500".to_string();
    let t = "10".to_string();
    //---------------------------------------------------

    let line1 = read(2, m.clone(), b.clone(), t.clone());
    println!("line1 = {}", line1);
    // render(line1, 6, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(3));

    let line2 = read(2, m.clone(), b.clone(), t.clone());
    println!("line2 = {}", line2);
    // render(line2, 1, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(3));

    let line3 = read(2, m.clone(), b.clone(), t.clone());
    println!("line3 = {}", line3);
    // render(line3, 2, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(2));

    let first = format!("{}{}{}", line1, line2, line3);
    react_no_leds(&first, 6, 1, 2);

    // special Earth rules
    // if first == "000" {
    //     thread::sleep(Duration::from_secs(2));
    //     render_yin(6, controller, &DEFAULT_COLOUR.to_string());
    //     render_yin(1, controller, &DEFAULT_COLOUR.to_string());
    //     render_yin(2, controller, &DEFAULT_COLOUR.to_string());
    // }

    // get related lines
    let lr1 = read(1, m.clone(), b.clone(), t.clone());
    let lr2 = read(1, m.clone(), b.clone(), t.clone());
    let lr3 = read(1, m.clone(), b.clone(), t.clone());

    drop_pins();
    thread::sleep(Duration::from_secs(3));
    //drop_li_to_default(controller);

    let line4 = read(2, m.clone(), b.clone(), t.clone());
    println!("line4 = {}", line4);
    // render(line4, 3, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(3));

    let line5 = read(2, m.clone(), b.clone(), t.clone());
    println!("line5 = {}", line5);
    // render(line5, 4, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(3));

    let line6 = read(2, m.clone(), b.clone(), t.clone());
    println!("line6 = {}", line6);
    // render(line6, 5, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(2));

    let second = format!("{}{}{}", line4, line5, line6);
    react_no_leds(&second, 3, 4, 5);

    // special Earth rules
    // if second == "000" {
    //     thread::sleep(Duration::from_secs(2));
    //     render_yin(3, controller, &DEFAULT_COLOUR.to_string());
    //     render_yin(4, controller, &DEFAULT_COLOUR.to_string());
    //     render_yin(5, controller, &DEFAULT_COLOUR.to_string());
    // }

    // get related lines
    let lr4 = read(1, m.clone(), b.clone(), t.clone());
    let lr5 = read(1, m.clone(), b.clone(), t.clone());
    let lr6 = read(1, m.clone(), b.clone(), t.clone());

    drop_pins();
    //drop_li_to_default(controller);

    let hexagram = format!("{}{}{}{}{}{}", line1, line2, line3, line4, line5, line6);
    let related_original = format!("{}{}{}{}{}{}", lr1, lr2, lr3, lr4, lr5, lr6);
    let related = get_related(&hexagram, &related_original);

    (hexagram, related)
}

// this is the main reading function
// first we read 3 lines with bias, threshold, multy parameters
// then we react depending on a first trigram
// then we read 3 related lines with a small time window
// then we read next 3 lines
// then we react depending on a second trigram
// then we read 3 related lines with a small time window
// from all this data we can build the hexagram and the related hexagram
pub fn reading(controller: &mut Controller) -> (String, String) {
    println!("New reading...");

    let yao = controller.leds_mut(0);
    for num in 0..LEDS_IN_LINE * 6 {
        yao[num as usize] = [0, 0, 0, 0];
    }
    if let Err(e) = controller.render() {
        println!("Yao error while reading: {:?}", e);
    };

    //---------------------------------------------------
    let m = "1".to_string();
    let b = "500".to_string();
    let t = "10".to_string();
    //---------------------------------------------------

    let line1 = read(2, m.clone(), b.clone(), t.clone());
    println!("line1 = {}", line1);
    render(line1, 6, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(3));

    let line2 = read(2, m.clone(), b.clone(), t.clone());
    println!("line2 = {}", line2);
    render(line2, 1, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(3));

    let line3 = read(2, m.clone(), b.clone(), t.clone());
    println!("line3 = {}", line3);
    render(line3, 2, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(2));

    let first = format!("{}{}{}", line1, line2, line3);
    react(controller, &first, 6, 1, 2);

    // special Earth rules
    if first == "000" {
        thread::sleep(Duration::from_secs(2));
        render_yin(6, controller, &DEFAULT_COLOUR.to_string());
        render_yin(1, controller, &DEFAULT_COLOUR.to_string());
        render_yin(2, controller, &DEFAULT_COLOUR.to_string());
    }

    // get related lines
    let lr1 = read(1, m.clone(), b.clone(), t.clone());
    let lr2 = read(1, m.clone(), b.clone(), t.clone());
    let lr3 = read(1, m.clone(), b.clone(), t.clone());

    drop_pins();
    thread::sleep(Duration::from_secs(3));
    //drop_li_to_default(controller);

    let line4 = read(2, m.clone(), b.clone(), t.clone());
    println!("line4 = {}", line4);
    render(line4, 3, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(3));

    let line5 = read(2, m.clone(), b.clone(), t.clone());
    println!("line5 = {}", line5);
    render(line5, 4, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(3));

    let line6 = read(2, m.clone(), b.clone(), t.clone());
    println!("line6 = {}", line6);
    render(line6, 5, controller, &DEFAULT_COLOUR.to_string());
    // render_li(controller);
    thread::sleep(Duration::from_secs(2));

    let second = format!("{}{}{}", line4, line5, line6);
    react(controller, &second, 3, 4, 5);

    // special Earth rules
    if second == "000" {
        thread::sleep(Duration::from_secs(2));
        render_yin(3, controller, &DEFAULT_COLOUR.to_string());
        render_yin(4, controller, &DEFAULT_COLOUR.to_string());
        render_yin(5, controller, &DEFAULT_COLOUR.to_string());
    }

    // get related lines
    let lr4 = read(1, m.clone(), b.clone(), t.clone());
    let lr5 = read(1, m.clone(), b.clone(), t.clone());
    let lr6 = read(1, m.clone(), b.clone(), t.clone());

    drop_pins();
    //drop_li_to_default(controller);

    let hexagram = format!("{}{}{}{}{}{}", line1, line2, line3, line4, line5, line6);
    let related_original = format!("{}{}{}{}{}{}", lr1, lr2, lr3, lr4, lr5, lr6);
    let related = get_related(&hexagram, &related_original);

    (hexagram, related)
}

// read pip data from the serial port
// install arduino ide + teense support to read from serial port on rpi
pub fn read_the_pip(delta: u64) -> Vec<i32> {
    let s = SerialPortSettings {
        baud_rate: 9600,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_secs(1),
    };

    let mut data: Vec<i32> = vec![];
    if let Ok(mut port) = serialport::open_with_settings("/dev/ttyACM0", &s) {
        let mut serial_buf: Vec<u8> = vec![0; 512];
        let start = SystemTime::now();
        loop {
            if let Ok(d) = start.elapsed() {
                if d > Duration::from_secs(delta) {
                    break;
                };
            }
            match port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    // println!("Pip val: {}", get_val(&serial_buf[..t]));
                    data.push(get_val(&serial_buf[..t]));
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    data
}

pub fn get_val(buf: &[u8]) -> i32 {
    let mut output = 0;
    let serial_data = std::str::from_utf8(buf).unwrap();
    if let Some(i) = serial_data.find("PiPVal: ") {
        let s = &serial_data[i + 8..];
        if let Some(j) = s.find("\r") {
            let str_value = &s[..j];
            if let Ok(value) = str_value.parse::<i32>() {
                output = value;
            }
        }
    }

    output
}

// read the pip data with timer and parameters
pub fn read(delta: u64, m: String, b: String, t: String) -> u8 {
    let _m: f32 = m.parse().unwrap_or_else(|_| 1.0);
    let b: f32 = b.parse().unwrap_or_else(|_| 0.0);
    let t: f32 = t.parse().unwrap_or_else(|_| 0.0);

    let data = read_the_pip(delta);
    println!("data: {:?}", data);

    let mut min = 0;
    if let Some(m) = data.iter().min() {
        min = *m;
    };
    println!("min: {}", min);

    let mut max = 0;
    if let Some(m) = data.iter().max() {
        max = *m;
    };
    println!("max: {}", max);

    let n_data = data.iter().map(|&i| i as f32 - b).collect::<Vec<f32>>();
    println!("n_data = {:?}", n_data);

    let mut mins: Vec<f32> = vec![];
    let mut maxs: Vec<f32> = vec![];
    for i in n_data.windows(3) {
        if i[1] > i[0] && i[1] > i[2] && i[1] > t {
            // println!("local max extremum = {:?}", i[1]);
            maxs.push(i[1]);
        }
        if i[1] < i[0] && i[1] < i[2] && i[1].abs() > t {
            // println!("local min extremum = {:?}", i[1]);
            mins.push(i[1]);
        }
        // println!("windows iter = {:?}", i);
    }

    // println!("mins = {:?}", mins);
    // println!("mins len = {:?}", mins.len());
    // println!("maxs = {:?}", maxs);
    // println!("maxs len = {:?}", maxs.len());

    if maxs.len() > mins.len() {
        1
    } else {
        0
    }
}

pub fn react_no_leds(trigram: &String, l1: i32, l2: i32, l3: i32) {
    match trigram.as_str() {
        // Heaven
        "111" => {
            pin_on(5);
            // render_yang(l1, controller, &HEAVEN_COLOUR.to_string());
            // render_yang(l2, controller, &HEAVEN_COLOUR.to_string());
            // render_yang(l3, controller, &HEAVEN_COLOUR.to_string());
        }
        // Cloud
        "110" => {
            pin_on(8);
            // render_yang(l1, controller, &CLOUD_COLOUR.to_string());
            // render_yang(l2, controller, &CLOUD_COLOUR.to_string());
            // render_yin(l3, controller, &CLOUD_COLOUR.to_string());
        }
        // Sun
        "101" => {
            shell_fire();
            // render_yang(l1, controller, &SUN_COLOUR.to_string());
            // render_yin(l2, controller, &SUN_COLOUR.to_string());
            // render_yang(l3, controller, &SUN_COLOUR.to_string());
        }
        // Wind
        "011" => {
            pin_on(20);
            // render_yin(l1, controller, &WIND_COLOUR.to_string());
            // render_yang(l2, controller, &WIND_COLOUR.to_string());
            // render_yang(l3, controller, &WIND_COLOUR.to_string());
        }
        // Thunder
        "100" => {
            play_sound("thunder.wav".to_string());
            // render_yang(l1, controller, &THUNDER_COLOUR.to_string());
            // render_yin(l2, controller, &THUNDER_COLOUR.to_string());
            // render_yin(l3, controller, &THUNDER_COLOUR.to_string());
        }
        // Water
        "010" => {
            pin_on(6);
            // render_yin(l1, controller, &WATER_COLOUR.to_string());
            // render_yang(l2, controller, &WATER_COLOUR.to_string());
            // render_yin(l3, controller, &WATER_COLOUR.to_string());
        }
        // Mountain
        "001" => {
            pin_on(7);
            play_sound("mountain.wav".to_string());
            // render_yin(l1, controller, &MOUNTAIN_COLOUR.to_string());
            // render_yin(l2, controller, &MOUNTAIN_COLOUR.to_string());
            // render_yang(l3, controller, &MOUNTAIN_COLOUR.to_string());
        }
        // Earth
        "000" => {
            play_sound("mountain.wav".to_string());
            // render_yin(l1, controller, &EARTH_COLOUR.to_string());
            // render_yin(l2, controller, &EARTH_COLOUR.to_string());
            // render_yin(l3, controller, &EARTH_COLOUR.to_string());
        }
        // Error
        _ => {}
    }
}

// here we react on tirgram with the hardware
pub fn react(controller: &mut Controller, trigram: &String, l1: i32, l2: i32, l3: i32) {
    match trigram.as_str() {
        // Heaven
        "111" => {
            pin_on(5);
            render_yang(l1, controller, &HEAVEN_COLOUR.to_string());
            render_yang(l2, controller, &HEAVEN_COLOUR.to_string());
            render_yang(l3, controller, &HEAVEN_COLOUR.to_string());
        }
        // Cloud
        "110" => {
            pin_on(8);
            render_yang(l1, controller, &CLOUD_COLOUR.to_string());
            render_yang(l2, controller, &CLOUD_COLOUR.to_string());
            render_yin(l3, controller, &CLOUD_COLOUR.to_string());
        }
        // Sun
        "101" => {
            shell_fire();
            render_yang(l1, controller, &SUN_COLOUR.to_string());
            render_yin(l2, controller, &SUN_COLOUR.to_string());
            render_yang(l3, controller, &SUN_COLOUR.to_string());
        }
        // Wind
        "011" => {
            pin_on(20);
            render_yin(l1, controller, &WIND_COLOUR.to_string());
            render_yang(l2, controller, &WIND_COLOUR.to_string());
            render_yang(l3, controller, &WIND_COLOUR.to_string());
        }
        // Thunder
        "100" => {
            play_sound("thunder.wav".to_string());
            render_yang(l1, controller, &THUNDER_COLOUR.to_string());
            render_yin(l2, controller, &THUNDER_COLOUR.to_string());
            render_yin(l3, controller, &THUNDER_COLOUR.to_string());
        }
        // Water
        "010" => {
            pin_on(6);
            render_yin(l1, controller, &WATER_COLOUR.to_string());
            render_yang(l2, controller, &WATER_COLOUR.to_string());
            render_yin(l3, controller, &WATER_COLOUR.to_string());
        }
        // Mountain
        "001" => {
            pin_on(7);
            play_sound("mountain.wav".to_string());
            render_yin(l1, controller, &MOUNTAIN_COLOUR.to_string());
            render_yin(l2, controller, &MOUNTAIN_COLOUR.to_string());
            render_yang(l3, controller, &MOUNTAIN_COLOUR.to_string());
        }
        // Earth
        "000" => {
            play_sound("mountain.wav".to_string());
            render_yin(l1, controller, &EARTH_COLOUR.to_string());
            render_yin(l2, controller, &EARTH_COLOUR.to_string());
            render_yin(l3, controller, &EARTH_COLOUR.to_string());
        }
        // Error
        _ => {}
    }
}

// turn the pins on and off on rpi model 4
pub fn pin_on(pin: u8) {
    println!("--------> pin {}: on", pin);

    if pin == 8 {
        pin8_start();
        // if let Ok(gpio) = Gpio::new() {
        //     if let Ok(pin8) = gpio.get(8) {
        //         let mut pin8 = pin8.into_output();
        //         pin8.set_high();
        //         thread::sleep(Duration::from_secs(6));
        //         pin8.set_low();
        //     }
        // }
    } else if pin == 7 {
        pin7_start();
        // if let Ok(gpio) = Gpio::new() {
        //     if let Ok(pin7) = gpio.get(7) {
        //         let mut pin7 = pin7.into_output();
        //         pin7.set_high();
        //         thread::sleep(Duration::from_secs(4));
        //         pin7.set_low();
        //     }
        // }
    } else {
        if let Ok(gpio) = Gpio::new() {
            if let Ok(pin) = gpio.get(pin) {
                let mut pin = pin.into_output();
                pin.set_high();
            }
        }
    }
    if pin == 6 || pin == 7 || pin == 8 {
        check_the_pumps();
    }
}

pub fn pin_off(pin: u8) {
    println!("--------> pin {}: off", pin);

    if let Ok(gpio) = Gpio::new() {
        if let Ok(pin) = gpio.get(pin) {
            let mut pin = pin.into_output();
            pin.set_low();
        }
    }
}

pub fn pin7_start() {
    println!("--------> pin7");

    if let Err(e) = process::Command::new("/ioracle/scripts/pin7.sh").output() {
        println!("pin7 error: {:?}", e);
    }
}

pub fn pin8_start() {
    println!("--------> pin8");

    if let Err(e) = process::Command::new("/ioracle/scripts/pin8.sh").output() {
        println!("pin8 error: {:?}", e);
    }
}

// experiments on fire launch
pub fn shell_fire() {
    println!("--------> fire");

    // if let Err(e) = process::Command::new("/ioracle/scripts/fire.sh").spawn() {
    //     println!("{:?}", e);
    // }
    if let Err(e) = process::Command::new("/ioracle/scripts/fire.sh").output() {
        println!("fire error: {:?}", e);
    }
    // process::Command::new("/ioracle/scripts/fire.sh")
    //     .output()
    //     .expect("");
}

// experiments on sounds
pub fn play_sound(file_name: String) {
    println!("--------> sound");

    let cmd;
    if file_name == "thunder.wav" {
        cmd = "/ioracle/scripts/thunder.sh";
    } else {
        cmd = "/ioracle/scripts/mountain.sh";
    }
    // process::Command::new(cmd).spawn().expect("");

    if let Err(e) = process::Command::new(cmd).output() {
        println!("sound error: {:?}", e);
    }
}

pub fn drop_pins() {
    println!("--------> drop pins");

    pin_off(5);
    // pin_off(8);
    pin_off(20);
    pin_off(6);
    // pin_off(7);
}

pub fn get_related(h: &String, r: &String) -> String {
    let mut result = "".to_string();
    for (x, y) in h.chars().zip(r.chars()) {
        if x.eq(&y) {
            if x.eq(&'0') {
                result = format!("{}1", result);
            } else {
                result = format!("{}0", result);
            }
        } else {
            result = format!("{}{}", result, x);
        }
    }

    result
}

fn parse_colour(colour: &String) -> (u8, u8, u8) {
    let mut str_buff = colour.clone();
    let mut rgb = (255, 255, 255);

    // colour string format:  "rgb(108, 73, 211)"
    let mut str_buff: String = str_buff.drain(4..).collect();
    str_buff.pop();
    let str_parts = str_buff.split(", ");
    let parts: Vec<&str> = str_parts.collect();

    if let Ok(part) = parts[0].parse::<u8>() {
        rgb.0 = part;
    }
    if let Ok(part) = parts[1].parse::<u8>() {
        rgb.1 = part;
    }
    if let Ok(part) = parts[2].parse::<u8>() {
        rgb.2 = part;
    }

    rgb
}

// check the pumps levels
fn check_the_pumps() {
    if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(PUMP_FILE) {
        let mut contents = String::new();
        if let Ok(_) = file.read_to_string(&mut contents) {
            if let Ok(num) = contents.parse::<i32>() {
                let mut x = num + 1;
                if x > 6 {
                    send_mail();
                    x = 0;
                }

                let xs = x.to_string();
                if let Ok(_) = file.seek(SeekFrom::Start(0)) {
                    if let Err(e) = file.write_all(&xs.as_bytes()) {
                        println!("{:?}", e);
                    };
                };
            }
        };
    } else {
        if let Ok(mut file) = File::create(PUMP_FILE) {
            if let Err(e) = file.write_all(b"1") {
                println!("{:?}", e);
            };
        };
    };
}

fn send_mail() {
    println!("refil the pumps!");
}
