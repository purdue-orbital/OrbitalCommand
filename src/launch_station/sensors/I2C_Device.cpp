#include <sys/ioctl.h>
#include <errno.h>
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <linux/i2c.h>
#include <linux/i2c-dev.h>
#include <syslog.h> // Logging
#include "I2C_Device.h"

I2C_Device::I2C_Device(int bus, int address) {
	_i2cbus = bus;
	_i2caddr = address;
	snprintf(busfile, sizeof(busfile), "/dev/i2c-%d", bus);
	openfd();
}

I2C_Device::~I2C_Device() {
	close(fd);
}

//! Read a single byte from I2C Bus
/*!
 \param address register address to read from
 */
uint8_t I2C_Device::read_byte(uint8_t address) {
	if (fd != -1) {
		uint8_t buff[BUFFER_SIZE];
		buff[0] = address;
		if (write(fd, buff, BUFFER_SIZE) != BUFFER_SIZE) {
			syslog(LOG_ERR,
					"I2C slave 0x%x failed to go to register 0x%x [read_byte():write %d]",
					_i2caddr, address, errno);
			return (-1);
		} else {
			if (read(fd, dataBuffer, BUFFER_SIZE) != BUFFER_SIZE) {
				syslog(LOG_ERR,
						"Could not read from I2C slave 0x%x, register 0x%x [read_byte():read %d]",
						_i2caddr, address, errno);
				return (-1);
			} else {
				return dataBuffer[0];
			}
		}
	} else {
		syslog(LOG_ERR, "Device File not available. Aborting read");
		return (-1);
	}

}

//! Read a block of bytes from I2C Bus
/*!
 \param address register address to start reading from
 \param num number of bytes to read
 */

uint8_t* I2C_Device::read_block(uint8_t address, int num) {
	uint8_t data[num];
	for(int i = 0; i < num; i++){
		data[i] = read_byte(address + i);
	}
	return data;
}

//! Write a single byte from a I2C Device
/*!
 \param address register address to write to
 \param data 8 bit data to write
 */
uint8_t I2C_Device::write_byte(uint8_t address, uint8_t data) {
	if (fd != -1) {
		uint8_t buff[2];
		buff[0] = address;
		buff[1] = data;
		if (write(fd, buff, sizeof(buff)) != 2) {
			syslog(LOG_ERR,
					"Failed to write to I2C Slave 0x%x @ register 0x%x [write_byte():write %d]",
					_i2caddr, address, errno);
			return (-1);
		} else {
			syslog(LOG_INFO, "Wrote to I2C Slave 0x%x @ register 0x%x [0x%x]",
					_i2caddr, address, data);
			return (-1);
		}
	} else {
		syslog(LOG_INFO, "Device File not available. Aborting write");
		return (-1);
	}
	return 0;
}
//! Open device file for I2C Device
void I2C_Device::openfd() {
	if ((fd = open(busfile, O_RDWR)) < 0) {
		syslog(LOG_ERR, "Couldn't open I2C Bus %d [openfd():open %d]", _i2cbus,
				errno);
	}
	if (ioctl(fd, I2C_SLAVE, _i2caddr) < 0) {
		syslog(LOG_ERR, "I2C slave %d failed [openfd():ioctl %d]", _i2caddr,
				errno);
	}
}