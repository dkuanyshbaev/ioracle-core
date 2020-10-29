use rand::distributions::{Distribution, Uniform};
use rppal::gpio::Gpio;
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};
use serialport::prelude::*;
use std::time::{Duration, SystemTime};
use std::{process, thread};

const LEDS_IN_LINE: i32 = 144;

const HEAVEN_COLOUR: &str = "rgb(224, 4, 235)";
const CLOUD_COLOUR: &str = "rgb(255, 2, 14)";
const SUN_COLOUR: &str = "rgb(255, 109, 0)";
const WIND_COLOUR: &str = "rgb(121, 255, 0)";
const THUNDER_COLOUR: &str = "rgb(255, 53, 6)";
const WATER_COLOUR: &str = "rgb(38, 2, 255)";
const MOUNTAIN_COLOUR: &str = "rgb(14, 255, 232)";
const EARTH_COLOUR: &str = "rgb(0, 0, 0)";

pub fn build_controller() -> Option<Controller> {
    match ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0,
            ChannelBuilder::new()
                .pin(12)
                .count(6 * LEDS_IN_LINE)
                .strip_type(StripType::Ws2811Rgb)
                .brightness(255)
                .build(),
        )
        .channel(
            1,
            ChannelBuilder::new()
                .pin(13)
                .count(3 * LEDS_IN_LINE)
                .strip_type(StripType::Ws2811Rgb)
                .brightness(255)
                .build(),
        )
        .build()
    {
        Ok(controller) => Some(controller),
        Err(_) => None,
    }
}

pub fn render(l: u8, line_num: i32, controller: &mut Controller, colour: &String) {
    match l {
        1 => render_yin(line_num, controller, colour),
        _ => render_yang(line_num, controller, colour),
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
            // leds[num as usize] = [a, b, c, 0];
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
        // leds[num as usize] = [a, b, c, 0];
        leds[num as usize] = [c, a, b, 0];
    }

    if let Err(e) = controller.render() {
        println!("{:?}", e);
    };
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

pub fn render_resting(controller: &mut Controller) {
    let mut rng1 = rand::thread_rng();
    let mut rng2 = rand::thread_rng();

    let yao = controller.leds_mut(0);
    let red_range = Uniform::from(54..255);

    let mut k;
    for i in 0..yao.len() - 1 {
        k = i * 9;
        // !!!???
        if k > yao.len() - 9 {
            k = yao.len() - 9;
        }
        for j in k..k + 9 {
            let r = red_range.sample(&mut rng1);
            let green_range = Uniform::from(0..r / 4);
            let g = green_range.sample(&mut rng2);
            yao[j as usize] = [0, g, r, 0];
        }
    }

    std::thread::sleep(Duration::from_millis(70));

    if let Err(e) = controller.render() {
        println!("Fire error: {:?}", e);
    }
}

pub fn reading(controller: &mut Controller) -> (String, String) {
    println!("New reading.");
    let yao = controller.leds_mut(0);

    for num in 0..LEDS_IN_LINE * 6 {
        yao[num as usize] = [0, 0, 0, 0];
    }

    if let Err(e) = controller.render() {
        println!("{:?}", e);
    };

    //---------------------------------------------------
    let m = "1".to_string();
    let b = "500".to_string();
    let t = "10".to_string();
    let default = "rgb(51, 0, 180)".to_string();
    //---------------------------------------------------

    let line1 = read(2, m.clone(), b.clone(), t.clone());
    println!("line1 = {}", line1);
    render(line1, 6, controller, &default);
    thread::sleep(Duration::from_secs(3));

    let line2 = read(2, m.clone(), b.clone(), t.clone());
    println!("line2 = {}", line2);
    render(line2, 1, controller, &default);
    thread::sleep(Duration::from_secs(3));

    let line3 = read(2, m.clone(), b.clone(), t.clone());
    println!("line3 = {}", line3);
    render(line3, 2, controller, &default);
    thread::sleep(Duration::from_secs(1));

    let first = format!("{}{}{}", line1, line2, line3);
    react(controller, &first, 6, 1, 2);

    // get related lines -----------------------------------
    let lr1 = read(1, m.clone(), b.clone(), t.clone());
    let lr2 = read(1, m.clone(), b.clone(), t.clone());
    let lr3 = read(1, m.clone(), b.clone(), t.clone());
    //------------------------------------------------------
    drop_pins();

    let line4 = read(2, m.clone(), b.clone(), t.clone());
    println!("line4 = {}", line4);
    render(line4, 3, controller, &default);
    thread::sleep(Duration::from_secs(3));

    let line5 = read(2, m.clone(), b.clone(), t.clone());
    println!("line5 = {}", line5);
    render(line5, 4, controller, &default);
    thread::sleep(Duration::from_secs(3));

    let line6 = read(2, m.clone(), b.clone(), t.clone());
    println!("line6 = {}", line6);
    render(line6, 5, controller, &default);
    thread::sleep(Duration::from_secs(1));

    let second = format!("{}{}{}", line4, line5, line6);
    react(controller, &second, 3, 4, 5);

    // get related lines -----------------------------------
    let lr4 = read(1, m.clone(), b.clone(), t.clone());
    let lr5 = read(1, m.clone(), b.clone(), t.clone());
    let lr6 = read(1, m.clone(), b.clone(), t.clone());
    //------------------------------------------------------
    drop_pins();

    let hexagram = format!("{}{}{}{}{}{}", line1, line2, line3, line4, line5, line6);
    let related_unchanged = format!("{}{}{}{}{}{}", lr1, lr2, lr3, lr4, lr5, lr6);
    let related = get_related(&hexagram, &related_unchanged);

    (hexagram, related)
}

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

    println!("mins = {:?}", mins);
    // println!("mins len = {:?}", mins.len());
    println!("maxs = {:?}", maxs);
    // println!("maxs len = {:?}", maxs.len());

    if maxs.len() > mins.len() {
        1
    } else {
        0
    }
}

// pub fn display(controller: &mut Controller, hexagram: &String, related: &String) {
pub fn display(hexagram: &String, related: &String) {
    let start = SystemTime::now();
    let first_color = get_trigram_colour(hexagram[0..3].to_string());
    let second_colour = get_trigram_colour(hexagram[3..6].to_string());
    let mut colour = "".to_string();

    loop {
        if let Ok(d) = start.elapsed() {
            if d > Duration::from_secs(120) {
                break;
            };
        }
        if let Some(mut controller) = build_controller() {
            let yao = controller.leds_mut(0);
            // let li = controller.leds_mut(1);

            // Yao
            let s = 3..=5;
            let mut i = 1;
            for (x, y) in hexagram.chars().zip(related.chars()) {
                if x.eq(&y) {
                    let mut b = 0;
                    if x.eq(&'1') {
                        b = 1;
                    }

                    if s.contains(&i) {
                        colour = second_colour.clone();
                    } else {
                        colour = first_color.clone();
                    }

                    render(b, i, &mut controller, &colour);
                } else {
                    // render_changing(i, controller);

                    // let position = LEDS_IN_LINE * (i - 1);
                    // for num in position..position + LEDS_IN_LINE {
                    //     // leds[num as usize] = [c, a, b, 0];
                    //     yao[num as usize] = [0, 0, 0, 0];
                    // }
                }
                i += 1;
            }

            // // Li
            // for num in 0..li.len() {
            //     li[num as usize] = [0, 0, 0, 0];
            // }

            if let Err(e) = controller.render() {
                println!("{:?}", e);
            };
        }
    }
}

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
            play_sound("Thunder.wav".to_string());
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
            play_sound("EarthMountain.wav".to_string());
            render_yin(l1, controller, &MOUNTAIN_COLOUR.to_string());
            render_yin(l2, controller, &MOUNTAIN_COLOUR.to_string());
            render_yang(l3, controller, &MOUNTAIN_COLOUR.to_string());
        }
        // Earth
        "000" => {
            play_sound("EarthMountain.wav".to_string());
            render_yin(l1, controller, &EARTH_COLOUR.to_string());
            render_yin(l2, controller, &EARTH_COLOUR.to_string());
            render_yin(l3, controller, &EARTH_COLOUR.to_string());
        }
        // Error
        _ => {}
    }
}

pub fn pin_on(pin: u8) {
    println!("--------> pin {}: on", pin);

    if pin == 8 {
        if let Ok(gpio) = Gpio::new() {
            if let Ok(pin8) = gpio.get(8) {
                let mut pin8 = pin8.into_output();
                pin8.set_high();
                thread::sleep(Duration::from_secs(6));
                pin8.set_low();
            }
        }
    } else if pin == 7 {
        if let Ok(gpio) = Gpio::new() {
            if let Ok(pin7) = gpio.get(7) {
                let mut pin7 = pin7.into_output();
                pin7.set_high();
                thread::sleep(Duration::from_secs(4));
                pin7.set_low();
            }
        }
    } else {
        if let Ok(gpio) = Gpio::new() {
            if let Ok(pin) = gpio.get(pin) {
                let mut pin = pin.into_output();
                pin.set_high();
            }
        }
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

pub fn shell_fire() {
    println!("--------> shell fire");

    match process::Command::new("/ioracle/scripts/fire.sh").output() {
        Ok(output) => println!("{:?}", output),
        Err(error) => println!("{:?}", error),
    }
}

pub fn play_sound(file_name: String) {
    println!("--------> play: {}", file_name);

    let cmd;
    if file_name == "Thunder.wav" {
        cmd = "/ioracle/scripts/thunder_sound.sh";
    } else {
        cmd = "/ioracle/scripts/earth_mountain.sh";
    }

    match process::Command::new(cmd).output() {
        Ok(output) => println!("{:?}", output),
        Err(error) => println!("{:?}", error),
    }
}

pub fn drop_pins() {
    println!("--------> drop pins");

    pin_off(5);
    pin_off(8);
    pin_off(20);
    pin_off(6);
    pin_off(7);
}

pub fn drop_lines(controller: &mut Controller) {
    println!("--------> drop lines");

    // all leds to resting_colour
    let (a, b, c) = parse_colour(&"rgb(2, 233, 211)".to_string());

    let yao_leds = controller.leds_mut(0);
    // for num in 0..yao_leds.len() - 1 {
    for num in 0..yao_leds.len() {
        yao_leds[num as usize] = [c, a, b, 0];
    }
    let li_leds = controller.leds_mut(1);
    // for num in 0..li_leds.len() - 1 {
    for num in 0..li_leds.len() {
        li_leds[num as usize] = [c, a, b, 0];
    }

    if let Err(error) = controller.render() {
        println!("{:?}", error);
    };
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

pub fn get_trigram_colour(t: String) -> String {
    match t.as_str() {
        // Heaven
        "111" => HEAVEN_COLOUR.to_string(),
        // Cloud
        "110" => CLOUD_COLOUR.to_string(),
        // Sun
        "101" => SUN_COLOUR.to_string(),
        // Wind
        "011" => WIND_COLOUR.to_string(),
        // Thunder
        "100" => THUNDER_COLOUR.to_string(),
        // Water
        "010" => WATER_COLOUR.to_string(),
        // Mountain
        "001" => MOUNTAIN_COLOUR.to_string(),
        // Earth
        "000" => EARTH_COLOUR.to_string(),
        // Error
        _ => "".to_string(),
    }
}
