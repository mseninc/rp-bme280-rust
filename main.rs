use rppal::i2c::I2c;

mod structs;
use structs::CalibParams;
use structs::EnvData;

// BME280 I2C default slave address.
const ADDR_BME280: u16 = 0x76;

// BME280 register addresses.
// cf. https://trac.switch-science.com/wiki/BME280
const REG_CTRL_HUM: usize = 0xF2;
const REG_CTRL_MEAS: usize = 0xF4;
const REG_CONFIG: usize = 0xF5;
const REG_CALIB_00: usize = 0x88;
const REG_CALIB_24: usize = 0xA1;
const REG_CALIB_26: usize = 0xE1;
const REG_CALIB_00_LEN: usize = 24;
const REG_CALIB_26_LEN: usize = 7;
const REG_ADC_VALUE: usize = 0xF7;
const REG_ADC_VALUE_LEN: usize = 8;

fn init_i2c() -> Result<I2c, rppal::i2c::Error> {

    let mut i2c = I2c::new()?;
    let osrs_t: u8 = 1; // Temperature oversampling x 1
    let osrs_p: u8 = 1; // Pressure oversampling x 1
    let osrs_h: u8 = 1; // Humidity oversampling x 1
    let mode: u8 = 3; // Normal mode
    let t_sb: u8 = 5; // Tstandby 1000ms
    let filter: u8 = 0; // Filter off
    let spi3w_en: u8 = 0; // 3-wire SPI Disable

    i2c.set_slave_address(ADDR_BME280)?;

    i2c.smbus_write_byte(
        REG_CTRL_HUM as u8,
        osrs_h as u8,
    )?;

    i2c.smbus_write_byte(
        REG_CTRL_MEAS as u8,
        ((osrs_t << 5) | (osrs_p << 2) | mode) as u8,
    )?;

    i2c.smbus_write_byte(
        REG_CONFIG as u8,
        ((t_sb << 5) | (filter << 2) | spi3w_en) as u8,
    )?;

    Ok(i2c)
}

fn read_calib(i2c: &I2c) -> Result<CalibParams, rppal::i2c::Error> {

    let mut calib = [0i32; 32];
    // 0x88 - 0x9F
    for n in 0..REG_CALIB_00_LEN {
        calib[n] = i2c.smbus_read_byte((REG_CALIB_00 as u8) + (n as u8))?.into();
    }
    // 0xA1
    calib[24] = i2c.smbus_read_byte(REG_CALIB_24 as u8)?.into();
    // 0xE1 - 0xE7
    for n in 0..REG_CALIB_26_LEN {
        calib[n + REG_CALIB_00_LEN + 1] = i2c.smbus_read_byte((REG_CALIB_26 as u8) + (n as u8))?.into();
    }

    let mut dig_t = [0i32; 3];
    dig_t[0] = (calib[1] << 8) | calib[0];
    dig_t[1] = (calib[3] << 8) | calib[2];
    dig_t[2] = (calib[5] << 8) | calib[4];

    let mut dig_p = [0i32; 9];
    dig_p[0] = (calib[7] << 8) | calib[6];
    dig_p[1] = (calib[9] << 8) | calib[8];
    dig_p[2] = (calib[11]<< 8) | calib[10];
    dig_p[3] = (calib[13]<< 8) | calib[12];
    dig_p[4] = (calib[15]<< 8) | calib[14];
    dig_p[5] = (calib[17]<< 8) | calib[16];
    dig_p[6] = (calib[19]<< 8) | calib[18];
    dig_p[7] = (calib[21]<< 8) | calib[20];
    dig_p[8] = (calib[23]<< 8) | calib[22];

    let mut dig_h = [0i32; 6];
    dig_h[0] = calib[24];
    dig_h[1] = (calib[26]<< 8) | calib[25];
    dig_h[2] = calib[27];
    dig_h[3] = (calib[28]<< 4) | (0x0F & calib[29]);
    dig_h[4] = (calib[30]<< 4) | ((calib[29] >> 4) & 0x0F);
    dig_h[5] = calib[31];

    // convert 1st-2nd params in dig_T as signed
    for n in 1..=2 {
        if dig_t[n] & 0x8000 == 0x8000 {
            dig_t[n] = (-dig_t[n] ^ 0xFFFF) + 1;
        }
    }
    // convert 1st-8th params in dig_P as signed
    for n in 1..=8 {
        if dig_p[n] & 0x8000 == 0x8000 {
            dig_p[n] = (-dig_p[n] ^ 0xFFFF) + 1;
        }
    }
    // convert 1, 3, 4 and 5th params in dig_H as signed
    for n in [1, 3, 4, 5].iter().map(|&x| x as usize) {
        if dig_h[n] & 0x8000 == 0x8000 {
            dig_h[n] = (-dig_h[n] ^ 0xFFFF) + 1;
        }
    }

    let result = CalibParams {
        temperature: dig_t,
        pressure: dig_p,
        humidity: dig_h,
    };

    Ok(result)
}

fn read_data(i2c: &I2c) -> Result<EnvData, rppal::i2c::Error> {

    let mut data = [0u32; 32];
    // 0xF7 - 0xFE
    for n in 0..REG_ADC_VALUE_LEN {
        data[n] = i2c.smbus_read_byte((REG_ADC_VALUE as u8) + (n as u8))?.into();
    }
    let result = EnvData {
        pressure: (data[0] << 12) | (data[1] << 4) | (data[2] >> 4),
        temperature: (data[3] << 12) | (data[4] << 4) | (data[5] >> 4),
        humidity: (data[6] << 8) | data[7],
    };

    Ok(result)
}

fn compute_temperature(calib_params: [i32; 3], raw_value: u32) -> f32 {
    let v1 = (raw_value as f32 / 16384.0 - calib_params[0] as f32 / 1024.0)
        * calib_params[1] as f32;
    let v2 = (raw_value as f32 / 131072.0 - calib_params[0] as f32 / 8192.0)
        * (raw_value as f32 / 131072.0 - calib_params[0] as f32 / 8192.0)
        * calib_params[2] as f32;
    return (v1 + v2) / 5120.0
}

fn compute_pressure(calib_params: [i32; 9], raw_value: u32, temperature: f32) -> f32 {
    let mut v1 = (temperature * 5120.0 / 2.0) - 64000.0;
    let mut v2 = (((v1 / 4.0) * (v1 / 4.0)) / 2048.0) * calib_params[5] as f32;
    v2 = v2 + ((v1 * calib_params[4] as f32) * 2.0);
    v2 = (v2 / 4.0) + (calib_params[3] as f32 * 65536.0);
    v1 = (((calib_params[2] as f32 * (((v1 / 4.0) * (v1 / 4.0)) / 8192.0)) / 8.0)
        + ((calib_params[1] as f32 * v1) / 2.0)) / 262144.0;
    v1 = ((32768.0 + v1) * calib_params[0] as f32) / 32768.0;
    if v1 == 0.0 {
        return 0.0
    }
    let pressure_tmp = ((1048576.0 - raw_value as f32) - (v2 / 4096.0)) * 3125.0;
    let pressure = (pressure_tmp as f32 * 2.0) / v1;
    v1 = (calib_params[8] as f32 * (((pressure / 8.0) * (pressure / 8.0)) / 8192.0)) / 4096.0;
    v2 = ((pressure / 4.0) * calib_params[7] as f32) / 8192.0;
    return (pressure + ((v1 + v2 + calib_params[6] as f32) / 16.0)) / 100.0
}

fn compute_humidity(calib_params: [i32; 6], raw_value: u32, temperature: f32) -> f32 {
    let mut h = temperature as f32 * 5120.0 - 76800.0;
    if h == 0.0 {
        return 0.0
    }
    h = (raw_value as f32 - (calib_params[3] as f32 * 64.0 + calib_params[4] as f32 / 16384.0 * h))
        * (calib_params[1] as f32 / 65536.0
            * (1.0 + calib_params[5] as f32 / 67108864.0 * h
                * (1.0 + calib_params[2] as f32 / 67108864.0 * h)
            )
        );
    h = h * (1.0 - calib_params[0] as f32 * h / 524288.0);
    return if h > 100.0 {
        100.0
    } else if h < 0.0 {
        0.0
    } else {
        h
    }
}

fn main() {
    let i2c = init_i2c();
    if let Ok(i2c) = i2c {
        let calib_data = read_calib(&i2c);
        if let Ok(calib_data) = calib_data {

            // for x in calib_data.temperature.iter() {
            //     println!("{}", x);
            // }
            // println!("");
            // for x in calib_data.pressure.iter() {
            //     println!("{}", x);
            // }
            // println!("");
            // for x in calib_data.humidity.iter() {
            //     println!("{}", x);
            // }
            let raw_data = read_data(&i2c);
            if let Ok(raw_data) = raw_data {
                // println!("{}", raw_data.temperature);
                // println!("{}", raw_data.pressure);
                // println!("{}", raw_data.humidity);
                let t = compute_temperature(calib_data.temperature, raw_data.temperature);
                println!("Temperature: {:.2} C", t);
                let h = compute_humidity(calib_data.humidity, raw_data.humidity, t);
                println!("Humidity: {:.2} %", h);
                let p = compute_pressure(calib_data.pressure, raw_data.pressure, t);
                println!("Pressure: {:.2} hPa", p);
            }
        }
    }
}
