rp-bme280-rust
===

"rp-bme280" is a simple data acquisition program from an ambient sensor Bosch BME280.
This works with Raspberry Pi 3+.

## Development environment

- Rust (1.44.1)

## Sensor

Refer to the pages below for the specification of the target sensor BME280.

- [Humidity Sensor BME280 | Bosch Sensortec](https://www.bosch-sensortec.com/products/environmental-sensors/humidity-sensors-bme280/)
- [Datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bme280-ds002.pdf)

## Features

- Data acquisition: Temperature (in degree Celsius), Humidity (in %), Pressure (in hPa)
- Acquisition data is compensated with the trimming parameters programmed in the devices' NVM during production

## Installation

"rp-bme280" uses the I2C interface on Raspberry Pi to communicate the sensor.

### Enabling I2C feature

Turn the I2C feature on in your Raspberry Pi. (The code below is for Raspberry Pi OS)

```sh
$ sudo raspi-config nonint do_i2c 0
```

- cf. [rc_gui/rc_gui.c at master · raspberrypi-ui/rc_gui](https://github.com/raspberrypi-ui/rc_gui/blob/master/src/rc_gui.c#L78)

### Connecting BME280 to Raspberry Pi

Connect BME280 communication pins to the default I2C port on your Raspberry Pi.

In [this module](https://www.amazon.co.jp/o/ASIN/B07LBCZZNM/) connections should be as the below.

### Downloading rp-bme280



## Usage

Just call the `rpbme280` executable.

```sh
$ /path/to/rpbme280
```

Then you can see the acquired data as the below.

```
Temperature: 25.96 C
Humidity: 52.52 %
Pressure: 1011.60 hPa
```

## Dependencies

- [golemparts/rppal: A Rust library that provides access to the Raspberry Pi's GPIO, I2C, PWM, SPI and UART peripherals.](https://github.com/golemparts/rppal)

## References

- [samplecodes/BME280 at master · SWITCHSCIENCE/samplecodes](https://github.com/SWITCHSCIENCE/samplecodes/tree/master/BME280)
- [BME280 – SWITCHSCIENCE](http://trac.switch-science.com/wiki/BME280)

## License

[MIT license](https://en.wikipedia.org/wiki/MIT_License)
